use std::{future::Future, time::Duration};

use tokio::time::sleep;

const INITIAL_BACKOFF_SECS: u64 = 2;
const MAX_BACKOFF_SECS: u64 = 1800;

pub async fn retry<T, E, F, Fut, P>(operation: F, max_retries: u32, should_retry_fn: Option<P>) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                let should_retry = match &should_retry_fn {
                    Some(predicate) => predicate(&error),
                    None => default_should_retry(&error),
                };
                if should_retry && attempt < max_retries {
                    attempt += 1;
                    sleep(retry_delay(attempt)).await;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

pub fn default_should_retry<E: std::fmt::Display>(error: &E) -> bool {
    let error_str = error.to_string().to_lowercase();

    error_str.contains("429")
        || error_str.contains("502")
        || error_str.contains("503")
        || error_str.contains("504")
        || error_str.contains("too many requests")
        || error_str.contains("throttled")
}

fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(INITIAL_BACKOFF_SECS.saturating_pow(attempt).min(MAX_BACKOFF_SECS))
}
