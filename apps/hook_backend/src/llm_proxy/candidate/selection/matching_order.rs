use time::OffsetDateTime;
use types::provider::{ProviderSchedulingMode, provider_key_minute_of_day};

use crate::llm_proxy::{AffinitySelection, cache::snapshot::CachedProviderKey};

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

pub(super) struct OrderedKeysInput<'a> {
    pub(super) keys: &'a [CachedProviderKey],
    pub(super) affinity: Option<&'a AffinitySelection>,
    pub(super) scheduling_mode: ProviderSchedulingMode,
    pub(super) request_id: &'a str,
}

pub(super) fn ordered_keys(input: OrderedKeysInput<'_>) -> Vec<CachedProviderKey> {
    let mut keys = input.keys.to_vec();
    match input.scheduling_mode {
        ProviderSchedulingMode::FixedOrder => {
            keys.sort_by(|left, right| (left.internal_priority, &left.id).cmp(&(right.internal_priority, &right.id)));
        }
        ProviderSchedulingMode::CacheAffinity => order_keys_for_cache_affinity(&mut keys, input.affinity, input.request_id),
        ProviderSchedulingMode::LoadBalance => order_keys_for_load_balance(&mut keys, input.request_id),
    }
    keys
}

pub(super) fn promote_affinity_endpoint<T, F>(endpoints: &mut Vec<T>, affinity: Option<&AffinitySelection>, endpoint_id: F)
where
    F: Fn(&T) -> &str,
{
    let Some(affinity) = affinity else {
        return;
    };
    let Some(index) = endpoints.iter().position(|endpoint| endpoint_id(endpoint) == affinity.endpoint_id) else {
        return;
    };
    let endpoint = endpoints.remove(index);
    endpoints.insert(0, endpoint);
}

pub(super) fn current_utc_minute() -> u16 {
    let time = OffsetDateTime::now_utc().time();
    provider_key_minute_of_day(u16::from(time.hour()), u16::from(time.minute())).expect("UTC time must have a valid minute of day")
}

fn order_keys_for_cache_affinity(keys: &mut Vec<CachedProviderKey>, affinity: Option<&AffinitySelection>, request_id: &str) {
    if let Some(affinity) = affinity {
        promote_affinity_key(keys, &affinity.key_id);
        return;
    }
    order_keys_for_load_balance(keys, request_id);
}

fn promote_affinity_key(keys: &mut Vec<CachedProviderKey>, key_id: &str) {
    let Some(index) = keys.iter().position(|key| key.id == key_id) else {
        return;
    };
    let key = keys.remove(index);
    keys.insert(0, key);
}

fn order_keys_for_load_balance(keys: &mut [CachedProviderKey], seed: &str) {
    keys.sort_by(|left, right| (stable_hash(&format!("{seed}:{}", left.id)), &left.id).cmp(&(stable_hash(&format!("{seed}:{}", right.id)), &right.id)));
}

fn stable_hash(value: &str) -> u64 {
    value
        .bytes()
        .fold(FNV_OFFSET_BASIS, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}
