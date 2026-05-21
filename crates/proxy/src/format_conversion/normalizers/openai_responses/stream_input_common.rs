use serde_json::Value;

pub(super) fn reserve_block_index(slot: &mut Option<u32>, next: &mut u32) -> u32 {
    if let Some(index) = *slot {
        return index;
    }
    let index = *next;
    *next = next.saturating_add(1);
    *slot = Some(index);
    index
}

pub(super) fn nested_string(value: Option<&Value>, key: &str) -> Option<String> {
    value?.get(key).and_then(Value::as_str).map(str::to_owned)
}
