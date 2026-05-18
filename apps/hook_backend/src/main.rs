mod auth;
mod commands;
mod llm_proxy;
mod migration;
mod performance_monitoring_api;
mod performance_monitoring_disk;
mod performance_monitoring_os;
mod performance_monitoring_tcp;
mod performance_monitoring_worker;
mod proxy_cache_hooks;
mod request_record_cleanup;
mod request_record_sweep;
mod startup;
mod system;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> BackendResult<()> {
    commands::run().await
}
