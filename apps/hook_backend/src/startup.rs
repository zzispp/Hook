use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    middleware,
};
use configuration::Settings;
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacService},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use storage::{DatabaseConnectOptions, connect_database};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router as create_user_router},
    application::UserService,
    infra::{Argon2PasswordHasher, ConfigSystemUserProvider, StorageUserRepository},
};

use crate::{
    BackendResult,
    auth::{AuthState, AuthStateParts, auth_middleware},
    system,
};

pub async fn serve(settings: Settings) -> BackendResult<()> {
    let state = build_app_state(&settings).await?;
    let app = create_app(state);
    let bind_addr = settings.bind_addr();
    let listener = TcpListener::bind(&bind_addr).await?;

    tracing::info!(addr = %bind_addr, "backend listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(
        &settings.database_url()?,
        DatabaseConnectOptions {
            push_schema: settings.database.push_schema_on_startup,
        },
    )
    .await?;
    let rbac = build_rbac_service(settings, database.clone()).await?;
    let users = Arc::new(UserService::with_system_user(
        StorageUserRepository::new(database),
        Argon2PasswordHasher,
        ConfigSystemUserProvider::from_settings(settings)?,
    ));
    let tokens = TokenService::new(token_settings(settings)?);
    let authorization = authorization_config(settings);

    Ok(AppState {
        users,
        tokens,
        rbac,
        authorization,
    })
}

async fn build_rbac_service(settings: &Settings, database: storage::Database) -> BackendResult<Arc<RbacService<StorageRbacRepository, RedisRbacCache>>> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let rbac = Arc::new(RbacService::new(repository, cache));

    crate::init::ensure_default_rbac(&rbac, settings).await?;
    rbac.rebuild_cache().await?;
    Ok(rbac)
}

fn create_app(state: AppState) -> Router {
    let user_state = ApiState::new(state.users.clone(), state.tokens.clone());
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac.clone());
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users,
        tokens: state.tokens,
        rbac: state.rbac,
        authorization: state.authorization,
    });
    let api_router = Router::new().merge(create_user_router(user_state)).merge(create_rbac_router(rbac_state));

    system::create_router()
        .nest("/api", api_router)
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:8082"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
}

fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: settings
            .auth
            .whitelist
            .iter()
            .map(|rule| AuthWhitelistRule {
                methods: rule.methods.clone(),
                path_pattern: rule.path_pattern.clone(),
            })
            .collect(),
    }
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
    authorization: AuthorizationConfig,
}
