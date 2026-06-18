use std::collections::HashSet;

use serde_json::Value;

use super::{
    is_call_item_type, is_call_output_item_type,
    json::{call_item_needs_cache, enrich_call_item_from_cache},
    non_empty_string, response_item_call_id,
    store::CodexChatHistoryStore,
    types::{CachedResponse, CodexChatHistoryEnrichment, CodexChatHistoryError, LookupResult},
};

pub(super) async fn enrich_request(store: &CodexChatHistoryStore, body: &mut Value) -> Result<CodexChatHistoryEnrichment, CodexChatHistoryError> {
    let previous_response_id = body.get("previous_response_id").and_then(non_empty_string).map(ToOwned::to_owned);
    let Some(input) = body.get_mut("input") else {
        return Ok(CodexChatHistoryEnrichment::default());
    };
    let original_input = std::mem::take(input);
    let original_was_object = matches!(original_input, Value::Object(_));
    let Some(items) = input_items(original_input, input) else {
        return Ok(CodexChatHistoryEnrichment::default());
    };
    let context = RequestCallContext::from_items(&items);
    if context.requested_call_ids.is_empty() {
        restore_input(input, items, original_was_object);
        return Ok(CodexChatHistoryEnrichment::default());
    }
    let lookup = store.lookup(previous_response_id.as_deref(), &context.requested_call_ids).await?;
    if let Err(error) = validate_lookup(previous_response_id, &lookup) {
        restore_input(input, items, original_was_object);
        return Err(error);
    }
    let outcome = enrich_items(items, original_was_object, &context, &lookup);
    *input = outcome.input;
    Ok(outcome.enrichment)
}

fn input_items(original_input: Value, input: &mut Value) -> Option<Vec<Value>> {
    match original_input {
        Value::Array(items) => Some(items),
        Value::Object(object) => Some(vec![Value::Object(object)]),
        other => {
            *input = other;
            None
        }
    }
}

fn validate_lookup(previous_response_id: Option<String>, lookup: &LookupResult) -> Result<(), CodexChatHistoryError> {
    if !lookup.ambiguous_call_ids.is_empty() {
        return Err(CodexChatHistoryError::Ambiguous {
            call_ids: lookup.ambiguous_call_ids.clone(),
        });
    }
    if !lookup.missing_call_ids.is_empty() {
        return Err(CodexChatHistoryError::Missing {
            previous_response_id,
            call_ids: lookup.missing_call_ids.clone(),
        });
    }
    Ok(())
}

struct RequestCallContext {
    output_call_ids: HashSet<String>,
    existing_call_ids: HashSet<String>,
    requested_call_ids: HashSet<String>,
}

impl RequestCallContext {
    fn from_items(items: &[Value]) -> Self {
        let output_call_ids = call_ids_by_kind(items, is_call_output_item_type);
        let existing_call_ids = call_ids_by_kind(items, is_call_item_type);
        let requested_call_ids = requested_call_ids(items, &output_call_ids, &existing_call_ids);
        Self {
            output_call_ids,
            existing_call_ids,
            requested_call_ids,
        }
    }
}

struct EnrichItemsOutcome {
    input: Value,
    enrichment: CodexChatHistoryEnrichment,
}

fn enrich_items(items: Vec<Value>, original_was_object: bool, context: &RequestCallContext, lookup: &LookupResult) -> EnrichItemsOutcome {
    let restore_group = restore_group(lookup, context);
    let restore_group_ids = restore_group.iter().map(|(call_id, _)| call_id.clone()).collect::<HashSet<_>>();
    let mut append_state = AppendState::new(restore_group, restore_group_ids);
    for item in items {
        append_state.append(item, lookup);
    }
    EnrichItemsOutcome {
        input: input_value(append_state.new_items, original_was_object),
        enrichment: append_state.enrichment,
    }
}

struct AppendState {
    restore_group: Option<Vec<(String, Value)>>,
    restore_group_ids: HashSet<String>,
    seen_call_ids: HashSet<String>,
    new_items: Vec<Value>,
    enrichment: CodexChatHistoryEnrichment,
}

impl AppendState {
    fn new(restore_group: Vec<(String, Value)>, restore_group_ids: HashSet<String>) -> Self {
        Self {
            restore_group: Some(restore_group),
            restore_group_ids,
            seen_call_ids: HashSet::new(),
            new_items: Vec::new(),
            enrichment: CodexChatHistoryEnrichment::default(),
        }
    }

    fn append(&mut self, item: Value, lookup: &LookupResult) {
        let Some(item_type) = item.get("type").and_then(Value::as_str) else {
            self.new_items.push(item);
            return;
        };
        if is_call_item_type(item_type) {
            self.append_call_item(item, lookup);
        } else if is_call_output_item_type(item_type) {
            self.append_call_output_item(item, lookup);
        } else {
            self.new_items.push(item);
        }
    }

    fn append_call_item(&mut self, mut item: Value, lookup: &LookupResult) {
        if let Some(call_id) = response_item_call_id(&item) {
            if let Some(cached) = lookup_call(lookup, &call_id)
                && enrich_call_item_from_cache(&mut item, cached)
            {
                self.enrichment.enriched_count += 1;
            }
            self.seen_call_ids.insert(call_id);
        }
        self.new_items.push(item);
    }

    fn append_call_output_item(&mut self, item: Value, lookup: &LookupResult) {
        self.append_restore_group();
        if let Some(call_id) = response_item_call_id(&item) {
            self.append_single_missing_call(&call_id, lookup);
        }
        self.new_items.push(item);
    }

    fn append_restore_group(&mut self) {
        let Some(group) = self.restore_group.take().filter(|group| !group.is_empty()) else {
            return;
        };
        for (call_id, cached_item) in group {
            self.seen_call_ids.insert(call_id);
            self.new_items.push(cached_item);
            self.enrichment.restored_count += 1;
        }
    }

    fn append_single_missing_call(&mut self, call_id: &str, lookup: &LookupResult) {
        if self.seen_call_ids.contains(call_id) || self.restore_group_ids.contains(call_id) {
            return;
        }
        if let Some(cached) = lookup_call(lookup, call_id).cloned() {
            self.seen_call_ids.insert(call_id.to_owned());
            self.new_items.push(cached);
            self.enrichment.restored_count += 1;
        }
    }
}

fn restore_group(lookup: &LookupResult, context: &RequestCallContext) -> Vec<(String, Value)> {
    let mut grouped_call_ids = HashSet::new();
    let mut group = Vec::new();
    if let Some(previous) = &lookup.previous {
        append_restore_group(previous, context, &mut grouped_call_ids, &mut group);
    }
    append_restore_group(&lookup.fallback, context, &mut grouped_call_ids, &mut group);
    group
}

fn append_restore_group(response: &CachedResponse, context: &RequestCallContext, grouped_call_ids: &mut HashSet<String>, group: &mut Vec<(String, Value)>) {
    for call_id in &response.call_order {
        if should_skip_restore(call_id, context, grouped_call_ids) {
            continue;
        }
        if let Some(item) = response.calls_by_id.get(call_id).cloned() {
            grouped_call_ids.insert(call_id.clone());
            group.push((call_id.clone(), item));
        }
    }
}

fn should_skip_restore(call_id: &str, context: &RequestCallContext, grouped_call_ids: &HashSet<String>) -> bool {
    !context.output_call_ids.contains(call_id) || context.existing_call_ids.contains(call_id) || grouped_call_ids.contains(call_id)
}

fn lookup_call<'a>(lookup: &'a LookupResult, call_id: &str) -> Option<&'a Value> {
    lookup
        .previous
        .as_ref()
        .and_then(|previous| previous.calls_by_id.get(call_id))
        .or_else(|| lookup.fallback.calls_by_id.get(call_id))
}

fn call_ids_by_kind(items: &[Value], kind: fn(&str) -> bool) -> HashSet<String> {
    items
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str).is_some_and(kind))
        .filter_map(response_item_call_id)
        .collect()
}

fn requested_call_ids(items: &[Value], output_call_ids: &HashSet<String>, existing_call_ids: &HashSet<String>) -> HashSet<String> {
    let missing_output_call_ids = output_call_ids.difference(existing_call_ids).cloned();
    let incomplete_call_ids = items.iter().filter_map(incomplete_call_item_id);
    missing_output_call_ids.chain(incomplete_call_ids).collect()
}

fn incomplete_call_item_id(item: &Value) -> Option<String> {
    let item_type = item.get("type").and_then(Value::as_str)?;
    if !is_call_item_type(item_type) || !call_item_needs_cache(item) {
        return None;
    }
    response_item_call_id(item)
}

fn restore_input(input: &mut Value, items: Vec<Value>, original_was_object: bool) {
    *input = input_value(items, original_was_object);
}

fn input_value(items: Vec<Value>, original_was_object: bool) -> Value {
    if original_was_object && items.len() == 1 {
        return items.into_iter().next().unwrap_or(Value::Null);
    }
    Value::Array(items)
}
