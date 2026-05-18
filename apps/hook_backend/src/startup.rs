use crate::{
    BackendResult,
    auth::{AuthState, AuthStateParts, auth_middleware},
    llm_proxy::{LlmProxyCache, LlmProxyState, cached_system_user_access, create_router as create_llm_proxy_router, create_v1beta_router},
    performance_monitoring_api::{PerformanceMonitoringApiState, create_router as create_performance_monitoring_router},
    performance_monitoring_os::PerformanceOsCollector,
    proxy_cache_hooks::{
        ProxyCachedApiTokenUseCase, ProxyCachedGroupUseCase, ProxyCachedModelUseCase, ProxyCachedProviderUseCase, ProxyCachedSettingUseCase,
        ProxyCachedUserUseCase,
    },
    system,
};
use api_token::{
    api::{ApiTokenApiState, create_router as create_api_token_router},
    application::ApiTokenService,
    infra::{StorageApiTokenRepository, StorageBillingGroupCatalog, StorageModelAccessCatalog, StorageSystemTokenPolicy, StorageUserCatalog},
};
use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware,
};
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
use configuration::{AuthWhitelistRule as ConfigAuthRule, Settings};
use dashboard::{
    api::{DashboardApiState, create_router as create_dashboard_router},
    application::DashboardService,
    infra::StorageDashboardRepository,
};
use group::{
    api::{GroupApiState, create_router as create_group_router},
    application::GroupService,
    infra::{StorageGroupModelCatalog, StorageGroupProviderCatalog, StorageGroupRepository},
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
use operations::{
    api::{OperationsApiState, create_router as create_operations_router},
    application::OperationsService,
    infra::{SmtpTicketMailer, StorageOperationsRepository},
};
use provider::{
    api::{ProviderApiState, create_router as create_provider_router},
    application::ProviderService,
    infra::{ProviderKeyCipher, ReqwestUpstreamModelFetcher, StorageGlobalModelCatalog, StorageProviderRepository},
};
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacService},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use setting::{
    api::{SettingApiState, create_router as create_setting_router},
    application::SettingService,
    infra::{LettreSmtpConnectionTester, SettingAesSecretCipher, StorageSettingRepository},
};
use std::{net::SocketAddr, sync::Arc};
use storage::connect_database;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use types::api_token::ApiTokenOwnerResponse;
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router as create_user_router},
    application::{SystemUserProvider, UserService},
    infra::{
        BcryptPasswordHasher, ConfigSystemUserProvider, SmtpPasswordResetMailer, StorageInitialGrantLedger, StoragePasswordResetConfig,
        StorageRegistrationPolicy, StorageUserRepository, StorageUserWalletCatalog,
    },
};
use wallet::{
    api::{WalletApiState, create_router as create_wallet_router},
    application::{SystemWalletProvider, WalletService},
    infra::{ConfigSystemWalletProvider, StorageWalletRepository},
};
pub async fn serve(settings: Settings) -> BackendResult<()> {
    let bind_addr = settings.bind_addr();
    hook_tracing::info_with_fields!("backend starting", addr = bind_addr);

    let state = build_app_state(&settings).await?;
    let app = create_app(state);
    let listener = TcpListener::bind(&bind_addr).await?;
    hook_tracing::info_with_fields!("backend listening", addr = bind_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    crate::request_record_cleanup::spawn_request_record_cleanup(database.clone());
    crate::request_record_sweep::spawn_request_record_sweep(database.clone());
    let performance_os_collector = Arc::new(PerformanceOsCollector::new()?);
    crate::performance_monitoring_worker::spawn_performance_monitoring_workers(database.clone(), performance_os_collector.clone());
    let rbac = build_rbac_service(settings, database.clone()).await?;
    let provider_key_cipher = ProviderKeyCipher::new(settings.provider_key_secret()?)?;
    let redis_connection = redis::Client::open(settings.redis_url()?)?.get_connection_manager().await?;
    let system_user_provider = ConfigSystemUserProvider::from_settings(settings)?;
    let system_wallet_provider = ConfigSystemWalletProvider::from_settings(settings)?;
    let proxy_cache = LlmProxyCache::new(
        database.clone(),
        redis_connection.clone(),
        settings.redis.key_prefix.clone(),
        cached_system_user_access(&system_user_provider),
    );
    proxy_cache.refresh_scheduling_snapshot().await?;
    proxy_cache.restore_provider_cooldowns().await?;
    let models_inner = Arc::new(ModelService::new(StorageModelRepository::new(database.clone()), ModelsDevClient::new()));
    let models = Arc::new(ProxyCachedModelUseCase::new(models_inner, proxy_cache.clone()));
    let providers_inner = Arc::new(ProviderService::new(
        StorageProviderRepository::new(database.clone()),
        StorageGlobalModelCatalog::new(database.clone()),
        provider_key_cipher.clone(),
        ReqwestUpstreamModelFetcher::new()?,
    ));
    let providers = Arc::new(ProxyCachedProviderUseCase::new(providers_inner, proxy_cache.clone()));
    let dashboard = Arc::new(DashboardService::new(StorageDashboardRepository::new(database.clone())));
    let wallets = Arc::new(WalletService::with_system_wallet(
        StorageWalletRepository::new(database.clone()),
        system_wallet_provider.clone(),
    ));
    let setting_secret_cipher = SettingAesSecretCipher::new(settings.provider_key_secret()?)?;
    let settings_inner = Arc::new(SettingService::new(
        StorageSettingRepository::new(database.clone()),
        setting_secret_cipher.clone(),
        LettreSmtpConnectionTester,
    ));
    let system_settings = Arc::new(ProxyCachedSettingUseCase::new(settings_inner, proxy_cache.clone()));
    let card_codes = Arc::new(CardCodeService::new(StorageCardCodeRepository::new(database.clone())));
    let groups_inner = Arc::new(GroupService::new(
        StorageGroupRepository::new(database.clone()),
        StorageGroupModelCatalog::new(database.clone()),
        StorageGroupProviderCatalog::new(database.clone()),
    ));
    let groups = Arc::new(ProxyCachedGroupUseCase::new(groups_inner, proxy_cache.clone()));
    let i18n = Arc::new(I18nService::new(StorageI18nRepository::new(database.clone())));
    let api_tokens_inner = Arc::new(ApiTokenService::new(
        StorageApiTokenRepository::new(database.clone()),
        StorageBillingGroupCatalog::new(database.clone()),
        StorageModelAccessCatalog::new(database.clone()),
        StorageUserCatalog::with_system_owner(database.clone(), api_token_system_owner(&system_user_provider)),
        StorageSystemTokenPolicy::new(database.clone()),
    ));
    let api_tokens = Arc::new(ProxyCachedApiTokenUseCase::new(api_tokens_inner, proxy_cache.clone()));
    let users_inner = Arc::new(UserService::with_system_user_and_registration(
        StorageUserRepository::new(database.clone()),
        BcryptPasswordHasher,
        system_user_provider,
        StorageRegistrationPolicy::new(database.clone()),
        StorageInitialGrantLedger::new(database.clone()),
        StorageUserWalletCatalog::new(database.clone()),
    )
    .with_password_reset(
        StoragePasswordResetConfig::new(database.clone()),
        SmtpPasswordResetMailer::new(database.clone(), setting_secret_cipher.clone()),
    ));
    let users = Arc::new(ProxyCachedUserUseCase::new(users_inner, proxy_cache.clone()));
    let operations = Arc::new(OperationsService::new(
        StorageOperationsRepository::new(database.clone()),
        SmtpTicketMailer::new(database.clone(), setting_secret_cipher),
        settings.admin.email.clone(),
    ));
    let captcha = Arc::new(CaptchaService::new(
        StorageCaptchaSettingsReader::new(database.clone()),
        RedisCaptchaStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
    ));
    let llm_proxy = LlmProxyState::new(
        database.clone(),
        provider_key_cipher,
        redis_connection,
        proxy_cache,
        settings.redis.key_prefix.clone(),
        system_wallet_provider.system_wallet().map(|record| record.wallet),
    );
    let tokens = TokenService::new(token_settings(settings)?);
    let authorization = authorization_config(settings);

    Ok(AppState {
        database,
        users,
        tokens,
        rbac,
        models,
        providers,
        dashboard,
        wallets,
        card_codes,
        system_settings,
        groups,
        i18n,
        api_tokens,
        operations,
        captcha,
        llm_proxy,
        performance_os_collector,
        authorization,
    })
}

fn api_token_system_owner(provider: &impl SystemUserProvider) -> Option<(String, ApiTokenOwnerResponse)> {
    let user = provider.system_user()?.user;
    Some((
        user.id.0,
        ApiTokenOwnerResponse {
            username: user.username,
            email: user.email,
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

fn create_app(state: AppState) -> Router {
    let user_state = ApiState::new(state.users.clone(), state.tokens.clone(), state.captcha.clone());
    let rbac_state = RbacApiState::new(state.authorization.clone(), state.rbac.clone(), state.rbac.clone());
    let model_state = ModelApiState::new(state.models);
    let provider_state = ProviderApiState::new(state.providers);
    let dashboard_state = DashboardApiState::new(state.dashboard);
    let wallet_state = WalletApiState::new(state.wallets);
    let card_code_state = CardCodeApiState::new(state.card_codes);
    let setting_state = SettingApiState::new(state.system_settings);
    let group_state = GroupApiState::new(state.groups);
    let i18n_state = I18nApiState::new(state.i18n);
    let api_token_state = ApiTokenApiState::new(state.api_tokens);
    let operations_state = OperationsApiState::new(state.operations);
    let captcha_state = CaptchaApiState::new(state.captcha);
    let performance_monitoring_state = PerformanceMonitoringApiState::new(state.database.clone(), state.performance_os_collector.clone());
    let llm_v1_router = create_llm_proxy_router(state.llm_proxy.clone());
    let gemini_router = create_v1beta_router(state.llm_proxy);
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users,
        tokens: state.tokens,
        rbac: state.rbac,
        authorization: state.authorization,
    });
    let api_router = Router::new()
        .merge(create_user_router(user_state))
        .merge(create_rbac_router(rbac_state))
        .merge(create_model_router(model_state))
        .merge(create_provider_router(provider_state))
        .merge(create_dashboard_router(dashboard_state))
        .merge(create_wallet_router(wallet_state))
        .merge(create_card_code_router(card_code_state))
        .merge(create_setting_router(setting_state))
        .merge(create_group_router(group_state))
        .merge(create_i18n_router(i18n_state))
        .merge(create_api_token_router(api_token_state))
        .merge(create_operations_router(operations_state))
        .merge(create_captcha_router(captcha_state))
        .merge(create_performance_monitoring_router(performance_monitoring_state));

    system::create_router()
        .nest("/v1", llm_v1_router)
        .nest("/v1beta", gemini_router)
        .nest("/api", api_router)
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
}
fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:8082"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}
fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: auth_rules(&settings.auth.whitelist),
        authenticated: auth_rules(&settings.auth.authenticated),
    }
}
fn auth_rules(rules: &[ConfigAuthRule]) -> Vec<AuthWhitelistRule> {
    rules
        .iter()
        .map(|rule| AuthWhitelistRule {
            methods: rule.methods.clone(),
            path_pattern: rule.path_pattern.clone(),
        })
        .collect()
}
fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
        access_token_ttl_seconds: settings.jwt.access_token_ttl_seconds,
        refresh_token_ttl_seconds: settings.jwt.refresh_token_ttl_seconds,
    })
}

struct AppState {
    database: storage::Database,
    users: Arc<dyn user::application::UserUseCase>,
    tokens: TokenService,
    rbac: Arc<RbacService<StorageRbacRepository, RedisRbacCache>>,
    models: Arc<dyn model::application::ModelUseCase>,
    providers: Arc<dyn provider::application::ProviderUseCase>,
    dashboard: Arc<dyn dashboard::application::DashboardUseCase>,
    wallets: Arc<dyn wallet::application::WalletUseCase>,
    card_codes: Arc<dyn card_code::application::CardCodeUseCase>,
    system_settings: Arc<dyn setting::application::SettingUseCase>,
    groups: Arc<dyn group::application::GroupUseCase>,
    i18n: Arc<dyn i18n::application::I18nUseCase>,
    api_tokens: Arc<dyn api_token::application::ApiTokenUseCase>,
    operations: Arc<dyn operations::application::OperationsUseCase>,
    captcha: Arc<dyn captcha::application::CaptchaUseCase>,
    llm_proxy: LlmProxyState,
    performance_os_collector: Arc<PerformanceOsCollector>,
    authorization: AuthorizationConfig,
}
