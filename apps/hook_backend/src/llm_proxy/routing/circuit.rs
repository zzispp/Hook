mod hash;
mod redis_ops;
mod rules;

use types::provider::RouteIdentity;

use crate::llm_proxy::{LlmProxyError, LlmProxyState};

use self::{
    hash::scope_hash,
    redis_ops::{circuit_key, exists, redis_error, set_ex, trim_failures, ttl, zcard},
    rules::{RoutingCircuitRule, default_rules},
};

const PROBE_SLOT_SECONDS: i64 = 30;
const HALF_OPEN_MARKER_SECONDS: i64 = 3_600;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CircuitCandidateState {
    Closed,
    HalfOpenProbe { reason: String },
    HalfOpenBusy { reason: String },
    Open { reason: String, ttl_seconds: i64 },
}

pub(crate) async fn candidate_state(state: &LlmProxyState, route: &RouteIdentity) -> Result<CircuitCandidateState, LlmProxyError> {
    let mut connection = state.affinity.clone();
    for rule in default_rules() {
        let hash = scope_hash(route, rule.scope);
        let open_key = circuit_key(state, "circuit", &hash, rule.id);
        let ttl_seconds = ttl(&mut connection, &open_key).await?;
        if ttl_seconds > 0 {
            return Ok(CircuitCandidateState::Open {
                reason: format!("circuit_open:{}", rule.id),
                ttl_seconds,
            });
        }
        if half_open_marker_exists(&mut connection, state, &hash, rule.id).await? {
            return half_open_state(&mut connection, state, &hash, rule.id).await;
        }
    }
    Ok(CircuitCandidateState::Closed)
}

pub(crate) async fn record_attempt(state: &LlmProxyState, event: CircuitAttemptEvent<'_>) -> Result<(), LlmProxyError> {
    let mut connection = state.affinity.clone();
    for rule in default_rules() {
        let hash = scope_hash(event.route, rule.scope);
        if event.success {
            close_half_open_if_ready(&mut connection, state, &hash, &rule).await?;
        }
        if rule.matches(&event)? {
            record_rule_failure(&mut connection, state, &hash, &rule, event.now_seconds).await?;
        }
    }
    Ok(())
}

pub(crate) struct CircuitAttemptEvent<'a> {
    pub(crate) route: &'a RouteIdentity,
    pub(crate) success: bool,
    pub(crate) status_code: Option<i32>,
    pub(crate) error_type: Option<&'a str>,
    pub(crate) error_code: Option<&'a str>,
    pub(crate) error_message: Option<&'a str>,
    pub(crate) now_seconds: i64,
}

async fn record_rule_failure(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule: &RoutingCircuitRule,
    now_seconds: i64,
) -> Result<(), LlmProxyError> {
    let failures_key = circuit_key(state, "failures", scope_hash, rule.id);
    let event_id = format!("{}:{now_seconds}", uuid::Uuid::now_v7());
    redis::cmd("ZADD")
        .arg(&failures_key)
        .arg(now_seconds)
        .arg(event_id)
        .query_async::<i64>(connection)
        .await
        .map_err(redis_error)?;
    trim_failures(connection, &failures_key, now_seconds - rule.window_seconds).await?;
    let observed = zcard(connection, &failures_key).await?;
    if observed >= rule.failure_count && observed >= rule.min_sample_count {
        open_circuit(connection, state, scope_hash, rule).await?;
    }
    Ok(())
}

async fn open_circuit(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule: &RoutingCircuitRule,
) -> Result<(), LlmProxyError> {
    let circuit = circuit_key(state, "circuit", scope_hash, rule.id);
    let marker = circuit_key(state, "opened", scope_hash, rule.id);
    set_ex(connection, &circuit, "open", rule.cooldown_seconds).await?;
    set_ex(connection, &marker, "1", rule.cooldown_seconds + HALF_OPEN_MARKER_SECONDS).await
}

async fn close_half_open_if_ready(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule: &RoutingCircuitRule,
) -> Result<(), LlmProxyError> {
    let marker = circuit_key(state, "opened", scope_hash, rule.id);
    if exists(connection, &marker).await? == 0 {
        return Ok(());
    }
    let successes = increment_probe_success(connection, state, scope_hash, rule.id).await?;
    if successes >= rule.half_open_success_count {
        close_rule(connection, state, scope_hash, rule.id).await?;
    }
    Ok(())
}

async fn increment_probe_success(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule_id: &str,
) -> Result<u64, LlmProxyError> {
    let success_key = circuit_key(state, "probe_success", scope_hash, rule_id);
    let successes = redis::cmd("INCR").arg(&success_key).query_async::<u64>(connection).await.map_err(redis_error)?;
    redis::cmd("EXPIRE")
        .arg(&success_key)
        .arg(HALF_OPEN_MARKER_SECONDS)
        .query_async::<i64>(connection)
        .await
        .map_err(redis_error)?;
    Ok(successes)
}

async fn close_rule(connection: &mut redis::aio::ConnectionManager, state: &LlmProxyState, scope_hash: &str, rule_id: &str) -> Result<(), LlmProxyError> {
    redis::cmd("DEL")
        .arg(circuit_key(state, "opened", scope_hash, rule_id))
        .arg(circuit_key(state, "probe_success", scope_hash, rule_id))
        .arg(circuit_key(state, "probe", scope_hash, rule_id))
        .arg(circuit_key(state, "failures", scope_hash, rule_id))
        .query_async::<i64>(connection)
        .await
        .map_err(redis_error)?;
    Ok(())
}

async fn half_open_state(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule_id: &str,
) -> Result<CircuitCandidateState, LlmProxyError> {
    let key = circuit_key(state, "probe", scope_hash, rule_id);
    let claimed = redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(PROBE_SLOT_SECONDS)
        .query_async::<Option<String>>(connection)
        .await
        .map_err(redis_error)?;
    Ok(if claimed.is_some() {
        CircuitCandidateState::HalfOpenProbe {
            reason: format!("circuit_half_open_probe:{rule_id}"),
        }
    } else {
        CircuitCandidateState::HalfOpenBusy {
            reason: format!("circuit_half_open_probe_busy:{rule_id}"),
        }
    })
}

async fn half_open_marker_exists(
    connection: &mut redis::aio::ConnectionManager,
    state: &LlmProxyState,
    scope_hash: &str,
    rule_id: &str,
) -> Result<bool, LlmProxyError> {
    Ok(exists(connection, &circuit_key(state, "opened", scope_hash, rule_id)).await? > 0)
}
