mod auth;
mod commands;
mod exchange_rates;
mod llm_proxy;
mod migration;
mod proxy_cache_hooks;
mod request_record_cleanup;
mod startup;
mod system;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    commands::run().await
}
