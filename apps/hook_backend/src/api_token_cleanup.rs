use std::{error::Error, time::Duration};

use api_token::{application::ApiTokenRepository, infra::StorageApiTokenRepository};
use storage::{Database, setting::SettingStore};

use crate::{llm_proxy::LlmProxyCache, proxy_cache_hooks::CachedApiTokenRepository};

const SECONDS_PER_MINUTE: u64 = 60;
const FALLBACK_RETRY_INTERVAL: Duration = Duration::from_secs(SECONDS_PER_MINUTE * 5);

type WorkerResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn spawn_api_token_cleanup(database: Database, cache: LlmProxyCache) {
    tokio::spawn(cleanup_loop(database, cache));
}

async fn cleanup_loop(database: Database, cache: LlmProxyCache) {
    loop {
        let interval = match run_and_log(database.clone(), cache.clone()).await {
            Ok(value) => value,
            Err(error) => {
                hook_tracing::error("api token cleanup failed", error.as_ref());
                FALLBACK_RETRY_INTERVAL
            }
        };
        tokio::time::sleep(interval).await;
    }
}

async fn run_and_log(database: Database, cache: LlmProxyCache) -> WorkerResult<Duration> {
    let settings = SettingStore::new(database.clone()).get_system_settings().await?;
    let interval = cleanup_interval(settings.token_expiry_check_interval_minutes);
    if !settings.auto_delete_expired_tokens {
        hook_tracing::info_with_fields!("api token cleanup skipped", reason = "auto cleanup disabled");
        return Ok(interval);
    }
    let deleted = run_cleanup(database, cache).await?;
    hook_tracing::info_with_fields!("api token cleanup completed", deleted_tokens = deleted);
    Ok(interval)
}

async fn run_cleanup(database: Database, cache: LlmProxyCache) -> WorkerResult<u64> {
    let repository = CachedApiTokenRepository::new(StorageApiTokenRepository::new(database), cache);
    repository.delete_expired_tokens().await.map_err(Into::into)
}

fn cleanup_interval(minutes: i64) -> Duration {
    Duration::from_secs(minutes as u64 * SECONDS_PER_MINUTE)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::cleanup_interval;

    #[test]
    fn cleanup_interval_uses_minutes() {
        assert_eq!(cleanup_interval(5), Duration::from_secs(300));
    }
}
