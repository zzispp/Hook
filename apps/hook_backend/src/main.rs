use std::sync::Arc;

use configuration::Settings;
use storage::connect_database;
use tokio::net::TcpListener;
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router},
    application::UserService,
    infra::{Argon2PasswordHasher, StorageUserRepository},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let settings = Settings::load().expect("failed to load YAML configuration");
    let database_url = settings.database_url().expect("failed to resolve database url");
    let database = connect_database(&database_url).await.expect("failed to connect database or push schema");
    let repository = StorageUserRepository::new(database);
    let users = UserService::new(repository, Argon2PasswordHasher);
    let tokens = TokenService::new(TokenSettings {
        secret: settings.jwt_secret().expect("failed to resolve jwt secret"),
        access_token_ttl_seconds: settings.jwt.access_token_ttl_seconds,
        refresh_token_ttl_seconds: settings.jwt.refresh_token_ttl_seconds,
    });
    let state = ApiState::new(Arc::new(users), tokens);
    let app = create_router(state);

    let bind_addr = settings.bind_addr();
    let listener = TcpListener::bind(&bind_addr).await.expect("failed to bind backend listener");
    tracing::info!(addr = %bind_addr, "backend listening");
    axum::serve(listener, app).await.expect("backend server stopped with error");
}
