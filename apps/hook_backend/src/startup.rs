use crate::{
    BackendResult,
    app_state::AppState,
    auth::{AuthState, AuthStateParts, auth_middleware},
    cache_monitoring_api::{CacheMonitoringApiState, create_router as create_cache_monitoring_router},
    frontend,
    http_config::{authorization_config, cors_layer, token_settings},
    llm_proxy::{
        LlmProxyCache, LlmProxyCacheOptions, LlmProxyProviderModelTester, LlmProxyState, cached_system_user_access, create_router as create_llm_proxy_router,
        create_v1beta_router,
    },
    model_status_probe::LlmProxyModelStatusProbe,
    performance_monitoring_api::{PerformanceMonitoringApiState, create_router as create_performance_monitoring_router},
    performance_monitoring_os::PerformanceOsCollector,
    proxy_cache_hooks::{
        CachedApiTokenRepository, CachedGroupRepository, CachedModelRepository, CachedProviderRepository, CachedSettingRepository, CachedUserRepository,
    },
    recharge_secret_cipher::RechargeAesSecretCipher,
    routing_api::{RoutingApiState, create_router as create_routing_router},
    system,
};
use api_token::{
    api::{ApiTokenApiState, create_router as create_api_token_router},
    application::ApiTokenService,
    infra::{StorageApiTokenRepository, StorageBillingGroupCatalog, StorageModelAccessCatalog, StorageSystemTokenPolicy, StorageUserCatalog},
};
use axum::{Router, middleware};
use captcha::{
    api::{CaptchaApiState, create_router as create_captcha_router},
    application::CaptchaService,
    infra::{RedisCaptchaStore, StorageCaptchaSettingsReader},
};
use card_code::{
    api::{CardCodeApiState, create_router as create_card_code_router},
    application::CardCodeService,
    infra::StorageCardCodeRepository,
};
use configuration::Settings;
use dashboard::{
    api::{DashboardApiState, create_router as create_dashboard_router},
    application::DashboardService,
    infra::StorageDashboardRepository,
};
use group::{
    api::{GroupApiState, create_router as create_group_router},
    application::GroupService,
    infra::{StorageGroupModelCatalog, StorageGroupProviderCatalog, StorageGroupRepository, StorageGroupUserGroupCatalog},
};
use i18n::{
    api::{I18nApiState, create_router as create_i18n_router},
    application::I18nService,
    infra::StorageI18nRepository,
};
use model::{
    api::{ModelApiState, create_router as create_model_router},
    application::ModelService,
    infra::{ModelsDevClient, StorageModelRepository},
};
use model_status::{
    api::{ModelStatusApiState, create_router as create_model_status_router},
    application::ModelStatusService,
    infra::{StorageModelStatusRepository, StorageModelStatusTokenCatalog},
};
use operations::{
    api::{OperationsApiState, create_router as create_operations_router},
    application::OperationsService,
    infra::{CaptchaTicketVerifier, SmtpTicketMailer, StorageOperationsRepository},
};
use payment::channels::EpayChannel;
use provider::{
    api::{ProviderApiState, create_router as create_provider_router},
    application::ProviderService,
    infra::{ProviderImportSource, ProviderKeyCipher, ReqwestUpstreamModelFetcher, StorageGlobalModelCatalog, StorageProviderRepository},
};
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::RbacService,
    infra::{RedisRbacCache, StorageRbacRepository},
};
use recharge::{
    api::{RechargeApiState, create_router as create_recharge_router},
    application::{PaymentChannelRegistry, RechargeService},
    infra::StorageRechargeRepository,
};
use scheduler::{
    api::{SchedulerApiState, create_router as create_scheduler_router},
    runtime::{SchedulerRuntime, SchedulerService},
};
use setting::{
    api::{SettingApiState, create_router as create_setting_router},
    application::SettingService,
    infra::{
        LettreSmtpConnectionTester, SettingAesSecretCipher, StorageSettingPaymentChannelCatalog, StorageSettingRepository, StorageSettingUserGroupCatalog,
    },
};
use std::{net::SocketAddr, sync::Arc};
use storage::{
    connect_database,
    provider::{ProviderStore, RequestPartitionMaintenanceOptions},
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use types::api_token::ApiTokenOwnerResponse;
use user::{
    api::{ApiState, TokenService, create_router as create_user_router},
    application::{SystemUserProvider, UserService},
    infra::{
        BcryptPasswordHasher, ConfigSystemUserProvider, RedisAuthTicketStore, RedisPurposeEmailCodeStore, RedisRegistrationEmailCodeStore, ReqwestOAuthClient,
        SmtpPasswordResetMailer, SmtpRegistrationEmailMailer, StorageAuthProviderConfig, StorageInitialGrantLedger, StoragePasswordResetConfig,
        StorageRegistrationEmailConfig, StorageRegistrationPolicy, StorageUserGroupBillingCatalog, StorageUserGroupRepository, StorageUserGroupSettingCatalog,
        StorageUserRepository, StorageUserWalletCatalog,
    },
};
use wallet::{
    api::{WalletApiState, create_router as create_wallet_router},
    application::{SystemWalletProvider, WalletService},
    infra::{ConfigSystemWalletProvider, StorageWalletRepository},
};

const REQUEST_RECORD_RETENTION_DAYS: i64 = 3;
const REQUEST_PAYLOAD_RETENTION_DAYS: i64 = 1;
const REQUEST_PARTITION_FUTURE_DAYS: i64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServeMode {
    Full,
    ApiOnly,
}

impl ServeMode {
    fn includes_frontend(self) -> bool {
        matches!(self, Self::Full)
    }
}

pub async fn serve(settings: Settings, mode: ServeMode) -> BackendResult<()> {
    if mode.includes_frontend() {
        frontend::ensure_assets()?;
    }

    let bind_addr = settings.bind_addr();
    hook_tracing::info_with_fields!("backend starting", addr = bind_addr);

    let state = build_app_state(&settings).await?;
    let app = create_app(state, mode);
    let listener = TcpListener::bind(&bind_addr).await?;
    hook_tracing::info_with_fields!("backend listening", addr = bind_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    ensure_request_record_partitions(&database).await?;
    let performance_os_collector = Arc::new(PerformanceOsCollector::new()?);
    let rbac = build_rbac_service(settings, database.clone()).await?;
    let provider_key_cipher = ProviderKeyCipher::new(settings.provider_key_secret()?)?;
    let redis_connection = redis::Client::open(settings.redis_url()?)?.get_connection_manager().await?;
    let system_user_provider = ConfigSystemUserProvider::from_settings(settings)?;
    let system_wallet_provider = ConfigSystemWalletProvider::from_settings(settings)?;
    let api_token_system_owner = api_token_system_owner(&system_user_provider);
    let proxy_cache = LlmProxyCache::new(LlmProxyCacheOptions {
        database: database.clone(),
        connection: redis_connection.clone(),
        key_prefix: settings.redis.key_prefix.clone(),
        system_users: cached_system_user_access(&system_user_provider),
        scheduling_snapshot_ttl_seconds: settings.redis.scheduling_snapshot_ttl_seconds,
    });
    proxy_cache.refresh_scheduling_snapshot().await?;
    proxy_cache.restore_provider_cooldowns().await?;
    let routing_metrics = crate::llm_proxy::routing::RoutingMetricsCache::load(database.clone()).await?;
    routing_metrics.spawn_refresh_loop();
    let models = Arc::new(ModelService::new(
        CachedModelRepository::new(StorageModelRepository::new(database.clone()), proxy_cache.clone()),
        ModelsDevClient::new(),
    ));
    let providers = Arc::new(ProviderService::new(
        CachedProviderRepository::new(StorageProviderRepository::new(database.clone()), proxy_cache.clone()),
        StorageGlobalModelCatalog::new(database.clone()),
        provider_key_cipher.clone(),
        ReqwestUpstreamModelFetcher::new()?,
        ProviderImportSource::new()?,
    ));
    let dashboard = Arc::new(DashboardService::new(StorageDashboardRepository::new(database.clone())));
    let wallets = Arc::new(WalletService::with_system_wallet(
        StorageWalletRepository::new(database.clone()),
        system_wallet_provider.clone(),
    ));
    let setting_secret_cipher = SettingAesSecretCipher::new(settings.provider_key_secret()?)?;
    let system_settings = Arc::new(
        SettingService::new(
            CachedSettingRepository::new(StorageSettingRepository::new(database.clone()), proxy_cache.clone()),
            setting_secret_cipher.clone(),
            LettreSmtpConnectionTester,
        )
        .with_user_group_catalog(StorageSettingUserGroupCatalog::new(database.clone()))
        .with_payment_channel_catalog(StorageSettingPaymentChannelCatalog::new(database.clone())),
    );
    let card_codes = Arc::new(CardCodeService::new(StorageCardCodeRepository::new(database.clone())));
    let payment_registry = PaymentChannelRegistry::with_providers(vec![Arc::new(EpayChannel)]);
    let payment_callback_endpoints = payment_registry.registered_callback_endpoints();
    let recharges = Arc::new(
        RechargeService::with_secret_cipher(
            StorageRechargeRepository::new(database.clone()),
            payment_registry.clone(),
            RechargeAesSecretCipher::new(setting_secret_cipher.clone()),
        )
        .await?,
    );
    let llm_proxy = LlmProxyState::new(
        database.clone(),
        provider_key_cipher,
        redis_connection.clone(),
        proxy_cache.clone(),
        routing_metrics,
        settings.redis.key_prefix.clone(),
        system_wallet_provider.system_wallet().map(|record| record.wallet),
    );
    let model_status = Arc::new(ModelStatusService::new(
        StorageModelStatusRepository::new(database.clone()),
        StorageModelStatusTokenCatalog::new(database.clone()),
        LlmProxyModelStatusProbe::new(llm_proxy.clone()),
    ));
    let scheduler_registry = Arc::new(crate::scheduled_tasks::scheduler_registry(
        proxy_cache.clone(),
        performance_os_collector.clone(),
        recharges.clone(),
        model_status.clone(),
        providers.clone(),
    )?);
    let scheduler_handle = SchedulerRuntime::spawn(database.clone(), scheduler_registry.clone())?;
    let scheduler = Arc::new(SchedulerService::new(
        storage::scheduler::SchedulerStore::new(database.clone()),
        scheduler_registry,
        scheduler_handle.clone(),
    ));
    let groups = Arc::new(GroupService::new(
        CachedGroupRepository::new(StorageGroupRepository::new(database.clone()), proxy_cache.clone()),
        StorageGroupModelCatalog::new(database.clone()),
        StorageGroupProviderCatalog::new(database.clone()),
        StorageGroupUserGroupCatalog::new(database.clone()),
    ));
    let i18n = Arc::new(I18nService::new(StorageI18nRepository::new(database.clone())));
    let api_tokens = Arc::new(ApiTokenService::new(
        CachedApiTokenRepository::new(StorageApiTokenRepository::new(database.clone()), proxy_cache.clone()),
        StorageBillingGroupCatalog::new(database.clone()),
        StorageModelAccessCatalog::new(database.clone()),
        StorageUserCatalog::with_system_owner(database.clone(), api_token_system_owner.clone()),
        StorageSystemTokenPolicy::new(database.clone()),
    ));
    let users = Arc::new(
        UserService::with_system_user_and_registration(
            CachedUserRepository::new(StorageUserRepository::new(database.clone()), proxy_cache.clone()),
            BcryptPasswordHasher,
            system_user_provider,
            StorageRegistrationPolicy::new(database.clone()),
            StorageInitialGrantLedger::new(database.clone()),
            StorageUserWalletCatalog::new(database.clone()),
        )
        .with_password_reset(
            StoragePasswordResetConfig::new(database.clone()),
            SmtpPasswordResetMailer::new(database.clone(), setting_secret_cipher.clone()),
        )
        .with_registration_email(
            StorageRegistrationEmailConfig::new(database.clone()),
            SmtpRegistrationEmailMailer::new(database.clone(), setting_secret_cipher.clone()),
            RedisRegistrationEmailCodeStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
        )
        .with_social_auth(
            StorageAuthProviderConfig::new(database.clone(), setting_secret_cipher.clone()),
            ReqwestOAuthClient::new()?,
            RedisAuthTicketStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
            RedisPurposeEmailCodeStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
        ),
    );
    let user_groups = Arc::new(user::application::UserGroupService::new(
        CachedUserRepository::new(StorageUserGroupRepository::new(database.clone()), proxy_cache.clone()),
        StorageUserGroupBillingCatalog::new(database.clone()),
        StorageUserGroupSettingCatalog::new(database.clone()),
    ));
    let captcha = Arc::new(CaptchaService::new(
        StorageCaptchaSettingsReader::new(database.clone()),
        RedisCaptchaStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
    ));
    let operations = Arc::new(OperationsService::new(
        StorageOperationsRepository::new(database.clone()),
        SmtpTicketMailer::new(database.clone(), setting_secret_cipher),
        CaptchaTicketVerifier::new(captcha.clone()),
        settings.admin.email.clone(),
    ));
    let tokens = TokenService::new(token_settings(settings)?);
    let authorization = authorization_config(settings, &payment_callback_endpoints);

    Ok(AppState {
        database,
        users: users.clone(),
        affiliates: users.clone(),
        admin_affiliates: users,
        user_groups,
        tokens,
        rbac,
        models,
        providers,
        dashboard,
        model_status,
        wallets,
        card_codes,
        recharges,
        system_settings,
        groups,
        i18n,
        api_tokens,
        cache_monitoring_system_owner: api_token_system_owner,
        operations,
        captcha,
        llm_proxy,
        performance_os_collector,
        scheduler,
        authorization,
    })
}

async fn ensure_request_record_partitions(database: &storage::Database) -> BackendResult<()> {
    ProviderStore::new(database.clone())
        .maintain_request_record_partitions(RequestPartitionMaintenanceOptions {
            now: time::OffsetDateTime::now_utc(),
            record_retention_days: REQUEST_RECORD_RETENTION_DAYS,
            payload_retention_days: REQUEST_PAYLOAD_RETENTION_DAYS,
            future_days: REQUEST_PARTITION_FUTURE_DAYS,
        })
        .await?;
    Ok(())
}

fn api_token_system_owner(provider: &impl SystemUserProvider) -> Option<(String, ApiTokenOwnerResponse)> {
    let user = provider.system_user()?.user;
    Some((
        user.id.0,
        ApiTokenOwnerResponse {
            username: user.username,
            email: user.email,
            group_codes: user.group_codes,
        },
    ))
}

async fn build_rbac_service(settings: &Settings, database: storage::Database) -> BackendResult<Arc<RbacService<StorageRbacRepository, RedisRbacCache>>> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let rbac = Arc::new(RbacService::new(repository, cache));

    rbac.rebuild_cache().await?;
    Ok(rbac)
}

fn create_app(state: AppState, mode: ServeMode) -> Router {
    let app = if mode.includes_frontend() {
        create_backend_router(&state).merge(frontend::create_router())
    } else {
        create_backend_router(&state)
    };

    app.layer(cors_layer()).layer(TraceLayer::new_for_http())
}

fn create_backend_router(state: &AppState) -> Router {
    let auth_state = create_auth_state(state);
    let api_router = create_api_router(state);
    let llm_v1_router = create_llm_proxy_router(state.llm_proxy.clone());
    let gemini_router = create_v1beta_router(state.llm_proxy.clone());

    system::create_router()
        .nest("/v1", llm_v1_router)
        .nest("/v1beta", gemini_router)
        .nest("/api", api_router)
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
}

fn create_auth_state(state: &AppState) -> AuthState {
    AuthState::new(AuthStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        authorization: state.authorization.clone(),
    })
}

fn create_api_router(state: &AppState) -> Router {
    let user_state = ApiState::new(
        state.users.clone(),
        state.affiliates.clone(),
        state.admin_affiliates.clone(),
        state.user_groups.clone(),
        state.tokens.clone(),
        state.captcha.clone(),
    );
    let rbac_state = RbacApiState::new(state.authorization.clone(), state.rbac.clone(), state.rbac.clone());
    let model_state = ModelApiState::new(state.models.clone());
    let provider_state = ProviderApiState::new(state.providers.clone(), Arc::new(LlmProxyProviderModelTester::new(state.llm_proxy.clone())));
    let dashboard_state = DashboardApiState::new(state.dashboard.clone());
    let model_status_state = ModelStatusApiState::new(state.model_status.clone());
    let wallet_state = WalletApiState::new(state.wallets.clone());
    let card_code_state = CardCodeApiState::new(state.card_codes.clone());
    let recharge_state = RechargeApiState::new(state.recharges.clone(), state.captcha.clone());
    let setting_state = SettingApiState::new(state.system_settings.clone());
    let group_state = GroupApiState::new(state.groups.clone());
    let i18n_state = I18nApiState::new(state.i18n.clone());
    let api_token_state = ApiTokenApiState::new(state.api_tokens.clone());
    let operations_state = OperationsApiState::new(state.operations.clone());
    let captcha_state = CaptchaApiState::new(state.captcha.clone());
    let performance_monitoring_state = PerformanceMonitoringApiState::new(state.database.clone(), state.performance_os_collector.clone());
    let cache_monitoring_state = CacheMonitoringApiState::new(state.database.clone(), state.llm_proxy.clone(), state.cache_monitoring_system_owner.clone());
    let scheduler_state = SchedulerApiState::new(state.scheduler.clone());
    let routing_state = RoutingApiState::new(state.llm_proxy.clone());

    Router::new()
        .merge(create_user_router(user_state))
        .merge(create_rbac_router(rbac_state))
        .merge(create_model_router(model_state))
        .merge(create_provider_router(provider_state))
        .merge(create_dashboard_router(dashboard_state))
        .merge(create_model_status_router(model_status_state))
        .merge(create_wallet_router(wallet_state))
        .merge(create_card_code_router(card_code_state))
        .merge(create_recharge_router(recharge_state))
        .merge(create_setting_router(setting_state))
        .merge(create_group_router(group_state))
        .merge(create_i18n_router(i18n_state))
        .merge(create_api_token_router(api_token_state))
        .merge(create_operations_router(operations_state))
        .merge(create_captcha_router(captcha_state))
        .merge(create_performance_monitoring_router(performance_monitoring_state))
        .merge(create_cache_monitoring_router(cache_monitoring_state))
        .merge(create_scheduler_router(scheduler_state))
        .merge(create_routing_router(routing_state))
}
