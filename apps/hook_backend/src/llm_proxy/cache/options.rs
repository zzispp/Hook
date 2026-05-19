use storage::Database;

use super::snapshot::CachedUserAccess;

pub struct LlmProxyCacheOptions {
    pub database: Database,
    pub connection: redis::aio::ConnectionManager,
    pub key_prefix: String,
    pub system_users: Vec<CachedUserAccess>,
    pub scheduling_snapshot_ttl_seconds: u64,
}
