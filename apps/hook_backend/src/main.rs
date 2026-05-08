mod auth;
mod commands;
mod migration;
mod startup;
mod system;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    tracing_subscriber::fmt::init();

    commands::run().await
}
