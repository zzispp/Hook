use std::collections::HashSet;

use serde_json::Value;

use super::{
    enrich, is_call_item_type, non_empty_string,
    redis_store::{RedisHistoryStore, history_infrastructure_error},
    response_item_call_id, sorted_call_ids,
    types::{CachedResponse, CodexChatHistoryEnrichment, CodexChatHistoryError, FallbackCall, LookupResult},
};
use crate::llm_proxy::LlmProxyError;

pub(super) const DEFAULT_HISTORY_TTL_SECONDS: u64 = 24 * 60 * 60;

#[derive(Clone, Debug)]
pub(crate) struct CodexChatHistoryStore {
    redis: RedisHistoryStore,
}

impl CodexChatHistoryStore {
    pub(crate) fn new(connection: redis::aio::ConnectionManager, key_prefix: impl Into<String>) -> Self {
        Self {
            redis: RedisHistoryStore::new(connection, key_prefix, DEFAULT_HISTORY_TTL_SECONDS),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_ttl_seconds(connection: redis::aio::ConnectionManager, key_prefix: impl Into<String>, ttl_seconds: u64) -> Self {
        Self {
            redis: RedisHistoryStore::new(connection, key_prefix, ttl_seconds),
        }
    }

    pub(crate) async fn record_response(&self, response: &Value) -> Result<usize, LlmProxyError> {
        let Some(response_id) = response_id(response) else {
            return Ok(0);
        };
        let calls = response_calls(response);
        if calls.is_empty() {
            return Ok(0);
        }
        self.insert_calls(&response_id, calls).await
    }

    pub(crate) async fn record_stream_event(&self, event: &Value) -> Result<usize, LlmProxyError> {
        match event.get("type").and_then(Value::as_str) {
            Some("response.output_item.done") => self.record_stream_call_item(event).await,
            Some("response.completed") => {
                let Some(response) = event.get("response") else {
                    return Ok(0);
                };
                self.record_response(response).await
            }
            _ => Ok(0),
        }
    }

    pub(crate) async fn enrich_request(&self, body: &mut Value) -> Result<CodexChatHistoryEnrichment, CodexChatHistoryError> {
        enrich::enrich_request(self, body).await
    }

    pub(super) async fn lookup(&self, previous_response_id: Option<&str>, requested_call_ids: &HashSet<String>) -> Result<LookupResult, CodexChatHistoryError> {
        let previous = self.cached_response(previous_response_id).await?;
        let mut missing_call_ids = Vec::new();
        let mut ambiguous_call_ids = Vec::new();
        let mut fallback_calls = Vec::new();
        for call_id in sorted_call_ids(requested_call_ids) {
            self.lookup_call_id(&call_id, previous.as_ref(), &mut fallback_calls, &mut missing_call_ids, &mut ambiguous_call_ids)
                .await?;
        }
        Ok(LookupResult {
            previous,
            fallback: fallback_response(fallback_calls),
            missing_call_ids,
            ambiguous_call_ids,
        })
    }

    async fn insert_calls(&self, response_id: &str, calls: Vec<(String, Value)>) -> Result<usize, LlmProxyError> {
        let existing = self.redis.read_response(response_id).await?;
        let mut response = existing.unwrap_or(CachedResponse {
            recorded_seq: self.redis.next_response_seq().await?,
            ..CachedResponse::default()
        });
        let mut inserted_or_updated = 0usize;
        let mut call_ids = Vec::with_capacity(calls.len());
        for (call_id, item) in calls {
            insert_call(&mut response, &call_id, item);
            call_ids.push(call_id);
            inserted_or_updated += 1;
        }
        self.redis.write_response(response_id, &response).await?;
        self.redis.index_calls(response_id, &call_ids).await?;
        self.redis.touch_seq_expiration().await?;
        Ok(inserted_or_updated)
    }

    async fn record_stream_call_item(&self, event: &Value) -> Result<usize, LlmProxyError> {
        let Some(response_id) = event.pointer("/response/id").and_then(non_empty_string).map(ToOwned::to_owned) else {
            return Ok(0);
        };
        let Some(call) = event.get("item").and_then(cached_call_item) else {
            return Ok(0);
        };
        self.insert_calls(&response_id, vec![call]).await
    }

    async fn cached_response(&self, response_id: Option<&str>) -> Result<Option<CachedResponse>, CodexChatHistoryError> {
        let Some(response_id) = response_id else {
            return Ok(None);
        };
        self.redis.read_response(response_id).await.map_err(history_infrastructure_error)
    }

    async fn lookup_call_id(
        &self,
        call_id: &str,
        previous: Option<&CachedResponse>,
        fallback_calls: &mut Vec<FallbackCall>,
        missing_call_ids: &mut Vec<String>,
        ambiguous_call_ids: &mut Vec<String>,
    ) -> Result<(), CodexChatHistoryError> {
        if previous.is_some_and(|response| response.calls_by_id.contains_key(call_id)) {
            return Ok(());
        }
        match self.unique_call(call_id).await? {
            UniqueCall::Found(call) => fallback_calls.push(call),
            UniqueCall::Missing => missing_call_ids.push(call_id.to_owned()),
            UniqueCall::Ambiguous => ambiguous_call_ids.push(call_id.to_owned()),
        }
        Ok(())
    }

    async fn unique_call(&self, call_id: &str) -> Result<UniqueCall, CodexChatHistoryError> {
        let response_ids = self.redis.call_response_ids(call_id).await?;
        let mut found = None;
        for response_id in response_ids {
            let Some(response) = self.redis.read_response(&response_id).await.map_err(history_infrastructure_error)? else {
                self.redis.remove_stale_index_entry(call_id, &response_id).await?;
                continue;
            };
            let Some(item) = response.calls_by_id.get(call_id).cloned() else {
                self.redis.remove_stale_index_entry(call_id, &response_id).await?;
                continue;
            };
            if found.is_some() {
                return Ok(UniqueCall::Ambiguous);
            }
            found = Some(FallbackCall {
                call_id: call_id.to_owned(),
                item,
                response_seq: response.recorded_seq,
                call_position: call_position(&response, call_id),
            });
        }
        Ok(found.map_or(UniqueCall::Missing, UniqueCall::Found))
    }
}

enum UniqueCall {
    Found(FallbackCall),
    Missing,
    Ambiguous,
}

fn response_calls(response: &Value) -> Vec<(String, Value)> {
    response
        .get("output")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(cached_call_item).collect())
        .unwrap_or_default()
}

fn insert_call(response: &mut CachedResponse, call_id: &str, item: Value) {
    if !response.calls_by_id.contains_key(call_id) {
        response.call_order.push(call_id.to_owned());
    }
    response.calls_by_id.insert(call_id.to_owned(), item);
}

fn cached_call_item(item: &Value) -> Option<(String, Value)> {
    let item_type = item.get("type").and_then(Value::as_str)?;
    if !is_call_item_type(item_type) {
        return None;
    }
    Some((response_item_call_id(item)?, item.clone()))
}

fn response_id(response: &Value) -> Option<String> {
    response.get("id").and_then(non_empty_string).map(ToOwned::to_owned)
}

fn fallback_response(mut calls: Vec<FallbackCall>) -> CachedResponse {
    calls.sort_by_key(|call| (call.response_seq, call.call_position));
    let mut response = CachedResponse::default();
    for call in calls {
        insert_call(&mut response, &call.call_id, call.item);
    }
    response
}

fn call_position(response: &CachedResponse, call_id: &str) -> usize {
    response.call_order.iter().position(|cached_id| cached_id == call_id).unwrap_or(usize::MAX)
}
