use std::sync::Arc;

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
use configuration::{AuthWhitelistRule as ConfigAuthRule, Settings};
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
use provider::{
    api::{ProviderApiState, create_router as create_provider_router},
    application::ProviderService,
    infra::{ProviderKeyCipher, StorageGlobalModelCatalog, StorageProviderRepository},
};
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacService},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use setting::{
    api::{SettingApiState, create_router as create_setting_router},
    application::SettingService,
    infra::StorageSettingRepository,
};
use storage::connect_database;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router as create_user_router},
    application::{SystemUserProvider, UserService},
    infra::{
        BcryptPasswordHasher, ConfigSystemUserProvider, StorageInitialGrantLedger, StorageRegistrationPolicy, StorageUserRepository, StorageUserWalletCatalog,
    },
};
use wallet::{
    api::{WalletApiState, create_router as create_wallet_router},
    application::WalletService,
    infra::StorageWalletRepository,
};

use crate::{
    BackendResult,
    auth::{AuthState, AuthStateParts, auth_middleware},
    exchange_rates::ExchangeRateCache,
    llm_proxy::{LlmProxyState, create_router as create_llm_proxy_router, create_v1beta_router},
    system,
};
use types::api_token::ApiTokenOwnerResponse;

pub async fn serve(settings: Settings) -> BackendResult<()> {
    let bind_addr = settings.bind_addr();
    hook_tracing::info_with_fields!("backend starting", addr = bind_addr);

    let state = build_app_state(&settings).await?;
    let app = create_app(state);
    let listener = TcpListener::bind(&bind_addr).await?;

    hook_tracing::info_with_fields!("backend listening", addr = bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    let exchange_rates = ExchangeRateCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    exchange_rates.clone().spawn_refresh_task();
    crate::request_record_cleanup::spawn_request_record_cleanup(database.clone());
    let rbac = build_rbac_service(settings, database.clone()).await?;
    let models = Arc::new(ModelService::new(StorageModelRepository::new(database.clone()), ModelsDevClient::new()));
    let provider_key_cipher = ProviderKeyCipher::new(settings.provider_key_secret()?)?;
    let providers = Arc::new(ProviderService::new(
        StorageProviderRepository::new(database.clone()),
        StorageGlobalModelCatalog::new(database.clone()),
        provider_key_cipher.clone(),
    ));
    let wallets = Arc::new(WalletService::new(StorageWalletRepository::new(database.clone())));
    let system_settings = Arc::new(SettingService::new(StorageSettingRepository::new(database.clone())));
    let groups = Arc::new(GroupService::new(
        StorageGroupRepository::new(database.clone()),
        StorageGroupModelCatalog::new(database.clone()),
        StorageGroupProviderCatalog::new(database.clone()),
    ));
    let i18n = Arc::new(I18nService::new(StorageI18nRepository::new(database.clone())));
    let system_user_provider = ConfigSystemUserProvider::from_settings(settings)?;
    let api_tokens = Arc::new(ApiTokenService::new(
        StorageApiTokenRepository::new(database.clone()),
        StorageBillingGroupCatalog::new(database.clone()),
        StorageModelAccessCatalog::new(database.clone()),
        StorageUserCatalog::with_system_owner(database.clone(), api_token_system_owner(&system_user_provider)),
        StorageSystemTokenPolicy::new(database.clone()),
    ));
    let users = Arc::new(UserService::with_system_user_and_registration(
        StorageUserRepository::new(database.clone()),
        BcryptPasswordHasher,
        system_user_provider,
        StorageRegistrationPolicy::new(database.clone()),
        StorageInitialGrantLedger::new(database.clone()),
        StorageUserWalletCatalog::new(database.clone()),
    ));
    let redis_connection = redis::Client::open(settings.redis_url()?)?.get_connection_manager().await?;
    let captcha = Arc::new(CaptchaService::new(
        StorageCaptchaSettingsReader::new(database.clone()),
        RedisCaptchaStore::new(redis_connection.clone(), settings.redis.key_prefix.clone()),
    ));
    let llm_proxy = LlmProxyState::new(
        database.clone(),
        StorageApiTokenRepository::new(database),
        provider_key_cipher,
        redis_connection,
        settings.redis.key_prefix.clone(),
    );
    let tokens = TokenService::new(token_settings(settings)?);
    let authorization = authorization_config(settings);

    Ok(AppState {
        users,
        tokens,
        rbac,
        models,
        providers,
        wallets,
        system_settings,
        groups,
        i18n,
        api_tokens,
        captcha,
        llm_proxy,
        exchange_rates,
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
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac.clone());
    let model_state = ModelApiState::new(state.models);
    let provider_state = ProviderApiState::new(state.providers);
    let wallet_state = WalletApiState::new(state.wallets);
    let setting_state = SettingApiState::new(state.system_settings, state.exchange_rates.clone());
    let group_state = GroupApiState::new(state.groups);
    let i18n_state = I18nApiState::new(state.i18n);
    let api_token_state = ApiTokenApiState::new(state.api_tokens);
    let captcha_state = CaptchaApiState::new(state.captcha);
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
        .merge(create_wallet_router(wallet_state))
        .merge(create_setting_router(setting_state))
        .merge(create_group_router(group_state))
        .merge(create_i18n_router(i18n_state))
        .merge(create_api_token_router(api_token_state))
        .merge(create_captcha_router(captcha_state));

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
    users: Arc<dyn user::application::UserUseCase>,
    tokens: TokenService,
    rbac: Arc<RbacService<StorageRbacRepository, RedisRbacCache>>,
    models: Arc<dyn model::application::ModelUseCase>,
    providers: Arc<dyn provider::application::ProviderUseCase>,
    wallets: Arc<dyn wallet::application::WalletUseCase>,
    system_settings: Arc<dyn setting::application::SettingUseCase>,
    groups: Arc<dyn group::application::GroupUseCase>,
    i18n: Arc<dyn i18n::application::I18nUseCase>,
    api_tokens: Arc<dyn api_token::application::ApiTokenUseCase>,
    captcha: Arc<dyn captcha::application::CaptchaUseCase>,
    llm_proxy: LlmProxyState,
    exchange_rates: ExchangeRateCache,
    authorization: AuthorizationConfig,
}
