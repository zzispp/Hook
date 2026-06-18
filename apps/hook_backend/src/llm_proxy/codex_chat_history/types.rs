use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(super) struct CachedResponse {
    pub(super) calls_by_id: HashMap<String, Value>,
    pub(super) call_order: Vec<String>,
    pub(super) recorded_seq: u64,
}

#[derive(Debug, Default)]
pub(super) struct LookupResult {
    pub(super) previous: Option<CachedResponse>,
    pub(super) fallback: CachedResponse,
    pub(super) missing_call_ids: Vec<String>,
    pub(super) ambiguous_call_ids: Vec<String>,
}

#[derive(Debug)]
pub(super) struct FallbackCall {
    pub(super) call_id: String,
    pub(super) item: Value,
    pub(super) response_seq: u64,
    pub(super) call_position: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CodexChatHistoryEnrichment {
    pub(crate) restored_count: usize,
    pub(crate) enriched_count: usize,
    pub(crate) missing_call_ids: Vec<String>,
    pub(crate) ambiguous_call_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CodexChatHistoryError {
    Infrastructure(String),
    Missing {
        previous_response_id: Option<String>,
        call_ids: Vec<String>,
    },
    Ambiguous {
        call_ids: Vec<String>,
    },
}

impl std::fmt::Display for CodexChatHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Infrastructure(message) => write!(f, "Codex chat history infrastructure error: {message}"),
            Self::Missing {
                previous_response_id,
                call_ids,
            } => write!(
                f,
                "missing Codex chat history for previous_response_id={} call_id_count={}",
                previous_response_id.as_deref().unwrap_or("<none>"),
                call_ids.len()
            ),
            Self::Ambiguous { call_ids } => write!(f, "ambiguous Codex chat history call_id_count={}", call_ids.len()),
        }
    }
}

impl std::error::Error for CodexChatHistoryError {}
