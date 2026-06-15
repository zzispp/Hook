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
mod routing_api;
mod scheduled_tasks;
mod startup;
mod system;

use tokio::runtime::Builder;

type BackendResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Prod-like routing audit and candidate persistence can exceed Tokio's default
// worker stack under real retry/failover paths, so reserve 4 MiB per worker.
const TOKIO_WORKER_STACK_SIZE_BYTES: usize = 4 * 1024 * 1024;

fn main() -> BackendResult<()> {
    Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(TOKIO_WORKER_STACK_SIZE_BYTES)
        .build()?
        .block_on(commands::run())
}
