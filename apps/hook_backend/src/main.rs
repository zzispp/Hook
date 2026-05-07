use std::sync::Arc;

use axum::{Router, middleware};
use configuration::Settings;
use rbac::{
    api::{RbacApiState, auth::auth_middleware, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacService},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use storage::{DatabaseConnectOptions, connect_database};
use tokio::net::TcpListener;
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router as create_user_router},
    application::UserService,
    infra::{Argon2PasswordHasher, ConfigSystemUserProvider, StorageUserRepository},
};

mod system;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    tracing_subscriber::fmt::init();

    let settings = Settings::load()?;
    match command_from_args(std::env::args().skip(1).collect())? {
        BackendCommand::Serve => serve(settings).await,
        BackendCommand::SchemaPush => push_schema(settings).await,
    }
}

async fn serve(settings: Settings) -> BackendResult<()> {
    let database_url = settings.database_url()?;
    let database = connect_database(
        &database_url,
        DatabaseConnectOptions {
            push_schema: settings.database.push_schema_on_startup,
        },
    )
    .await?;
    let user_repository = StorageUserRepository::new(database.clone());
    let rbac_repository = StorageRbacRepository::new(database);
    let rbac_cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let rbac = Arc::new(RbacService::new(rbac_repository, rbac_cache));
    rbac.rebuild_cache().await?;
    let users = Arc::new(UserService::with_system_user(
        user_repository,
        Argon2PasswordHasher,
        ConfigSystemUserProvider::from_settings(&settings)?,
    ));
    let tokens = TokenService::new(TokenSettings {
        secret: settings.jwt_secret()?,
        access_token_ttl_seconds: settings.jwt.access_token_ttl_seconds,
        refresh_token_ttl_seconds: settings.jwt.refresh_token_ttl_seconds,
    });
    let user_state = ApiState::new(users.clone(), tokens.clone());
    let rbac_state = RbacApiState::new(users, tokens, rbac.clone(), rbac, authorization_config(&settings));
    let api_router = Router::new()
        .merge(create_user_router(user_state))
        .merge(create_rbac_router(rbac_state.clone()));
    let app = system::create_router()
        .nest("/api", api_router)
        .layer(middleware::from_fn_with_state(rbac_state, auth_middleware));

    let bind_addr = settings.bind_addr();
    let listener = TcpListener::bind(&bind_addr).await?;
    tracing::info!(addr = %bind_addr, "backend listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn push_schema(settings: Settings) -> BackendResult<()> {
    let database_url = settings.database_url()?;
    let database = connect_database(&database_url, DatabaseConnectOptions { push_schema: false }).await?;

    database.push_schema().await?;
    tracing::info!("database schema pushed");
    Ok(())
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackendCommand {
    Serve,
    SchemaPush,
}

fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let positionals = positional_args(args)?;
    match positionals.as_slice() {
        [] => Ok(BackendCommand::Serve),
        [schema, push] if schema == "schema" && push == "push" => Ok(BackendCommand::SchemaPush),
        _ => Err(format!("unsupported backend command: {}", positionals.join(" ")).into()),
    }
}

fn positional_args(args: Vec<String>) -> BackendResult<Vec<String>> {
    let mut positionals = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--config" {
            args.next().ok_or("--config requires a file path")?;
            continue;
        }
        positionals.push(arg);
    }

    Ok(positionals)
}

#[cfg(test)]
mod tests {
    use super::{BackendCommand, command_from_args, positional_args};

    #[test]
    fn defaults_to_serve_command() {
        assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
    }

    #[test]
    fn detects_schema_push_command() {
        let args = vec!["schema".into(), "push".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::SchemaPush);
    }

    #[test]
    fn ignores_config_path_when_detecting_command() {
        let args = vec!["--config".into(), "config/config.yaml".into(), "schema".into(), "push".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::SchemaPush);
    }

    #[test]
    fn rejects_unknown_command() {
        let args = vec!["schem".into(), "push".into()];

        assert!(command_from_args(args).is_err());
    }

    #[test]
    fn rejects_missing_config_path() {
        let args = vec!["--config".into()];

        assert!(positional_args(args).is_err());
    }
}
