use crate::llm_proxy::{LlmProxyError, LlmProxyState};

pub(super) async fn trim_failures(connection: &mut redis::aio::ConnectionManager, key: &str, min_score: i64) -> Result<(), LlmProxyError> {
    redis::cmd("ZREMRANGEBYSCORE")
        .arg(key)
        .arg("-inf")
        .arg(min_score)
        .query_async::<i64>(connection)
        .await
        .map_err(redis_error)?;
    Ok(())
}

pub(super) async fn zcard(connection: &mut redis::aio::ConnectionManager, key: &str) -> Result<u64, LlmProxyError> {
    redis::cmd("ZCARD").arg(key).query_async(connection).await.map_err(redis_error)
}

pub(super) async fn ttl(connection: &mut redis::aio::ConnectionManager, key: &str) -> Result<i64, LlmProxyError> {
    redis::cmd("TTL").arg(key).query_async(connection).await.map_err(redis_error)
}

pub(super) async fn exists(connection: &mut redis::aio::ConnectionManager, key: &str) -> Result<i64, LlmProxyError> {
    redis::cmd("EXISTS").arg(key).query_async(connection).await.map_err(redis_error)
}

pub(super) async fn set_ex(connection: &mut redis::aio::ConnectionManager, key: &str, value: &str, seconds: i64) -> Result<(), LlmProxyError> {
    redis::cmd("SETEX")
        .arg(key)
        .arg(seconds)
        .arg(value)
        .query_async::<String>(connection)
        .await
        .map_err(redis_error)?;
    Ok(())
}

pub(super) fn circuit_key(state: &LlmProxyState, kind: &str, scope_hash: &str, rule_id: &str) -> String {
    format!("{}:llm_proxy:routing:{kind}:{scope_hash}:{rule_id}", state.key_prefix)
}

pub(super) fn redis_error(error: redis::RedisError) -> LlmProxyError {
    LlmProxyError::Infrastructure(format!("redis error: {error}"))
}
