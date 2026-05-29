mod app_state;
mod auth;
mod cache_monitoring_api;
mod commands;
mod frontend;
mod http_config;
mod llm_proxy;
mod migration;
mod model_status_probe;
mod performance_monitoring_api;
mod performance_monitoring_disk;
mod performance_monitoring_os;
mod performance_monitoring_tcp;
mod proxy_cache_hooks;
mod recharge_secret_cipher;
mod scheduled_tasks;
mod startup;
mod system;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    commands::run().await
}
