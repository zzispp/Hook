use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 15;
const DEFAULT_POOL_IDLE_TIMEOUT_SECS: u64 = 90;
const DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 20;
const DEFAULT_TCP_KEEPALIVE_SECS: u64 = 60;

pub fn builder() -> reqwest::ClientBuilder {
    base_builder().timeout(default_timeout())
}

pub fn long_stream_builder() -> reqwest::ClientBuilder {
    base_builder()
}

pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}

fn base_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS))
        .pool_idle_timeout(Duration::from_secs(DEFAULT_POOL_IDLE_TIMEOUT_SECS))
        .pool_max_idle_per_host(DEFAULT_POOL_MAX_IDLE_PER_HOST)
        .tcp_keepalive(Duration::from_secs(DEFAULT_TCP_KEEPALIVE_SECS))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::default_timeout;

    #[test]
    fn default_timeout_is_30_seconds() {
        assert_eq!(default_timeout(), Duration::from_secs(30));
    }
}
