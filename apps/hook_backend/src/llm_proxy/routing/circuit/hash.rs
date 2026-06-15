use types::provider::RouteIdentity;

use super::rules::CircuitScope;

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

pub(super) fn scope_hash(route: &RouteIdentity, scope: CircuitScope) -> String {
    let value = match scope {
        CircuitScope::Route => route_key(route),
        CircuitScope::Key => format!("key:{}:{}", route.provider_id, route.key_id),
    };
    format!("{:016x}", stable_hash(&value))
}

fn route_key(route: &RouteIdentity) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        route.provider_id, route.key_id, route.endpoint_id, route.global_model_id, route.client_api_format, route.provider_api_format, route.is_stream
    )
}

fn stable_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}
