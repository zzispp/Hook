use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::application::{ProviderError, ProviderResult, UpstreamImportModel};

const MAX_ERROR_BODY_CHARS: usize = 300;

pub(super) type GroupMap = BTreeMap<String, NewApiGroup>;

#[derive(Debug, Deserialize)]
pub(super) struct TokenListEnvelope {
    pub(super) data: TokenListData,
    #[serde(default)]
    message: String,
    success: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct TokenListData {
    pub(super) total: u64,
    pub(super) items: Vec<NewApiTokenRecord>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NewApiTokenRecord {
    pub(super) id: i64,
    pub(super) key: String,
    pub(super) status: i32,
    pub(super) name: String,
    pub(super) group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct NewApiGroup {
    #[serde(with = "rust_decimal::serde::float")]
    ratio: Decimal,
}

impl NewApiGroup {
    pub(super) fn ratio(&self) -> Decimal {
        self.ratio
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct GroupsEnvelope {
    pub(super) data: GroupMap,
    #[serde(default)]
    message: String,
    success: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct TokenKeyEnvelope {
    pub(super) data: TokenKeyData,
    #[serde(default)]
    message: String,
    success: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct TokenKeyData {
    pub(super) key: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ModelsEnvelope {
    pub(super) data: Vec<NewApiModelRecord>,
    #[serde(default)]
    message: String,
    success: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct NewApiModelRecord {
    id: String,
    #[serde(default)]
    supported_endpoint_types: Vec<String>,
}

pub(super) trait NewApiEnvelope {
    fn success(&self) -> bool;
    fn message(&self) -> &str;
}

impl NewApiEnvelope for TokenListEnvelope {
    fn success(&self) -> bool {
        self.success
    }

    fn message(&self) -> &str {
        &self.message
    }
}

impl NewApiEnvelope for GroupsEnvelope {
    fn success(&self) -> bool {
        self.success
    }

    fn message(&self) -> &str {
        &self.message
    }
}

impl NewApiEnvelope for TokenKeyEnvelope {
    fn success(&self) -> bool {
        self.success
    }

    fn message(&self) -> &str {
        &self.message
    }
}

impl NewApiEnvelope for ModelsEnvelope {
    fn success(&self) -> bool {
        self.success
    }

    fn message(&self) -> &str {
        &self.message
    }
}

pub(super) fn decode_envelope<T>(text: &str) -> ProviderResult<T>
where
    T: for<'de> Deserialize<'de> + NewApiEnvelope,
{
    let envelope: T = serde_json::from_str(text).map_err(json_error)?;
    if envelope.success() {
        return Ok(envelope);
    }
    Err(ProviderError::Infrastructure(format!("newapi returned failure: {}", envelope.message())))
}

pub(super) fn model_response(record: NewApiModelRecord) -> UpstreamImportModel {
    UpstreamImportModel {
        id: record.id,
        supported_endpoint_types: record.supported_endpoint_types,
    }
}

pub(super) fn token_group_ratio(groups: &GroupMap, group: Option<&str>) -> ProviderResult<Decimal> {
    let Some(group) = group else {
        return Err(ProviderError::Infrastructure("newapi token group is missing".into()));
    };
    groups
        .get(group)
        .map(|item| item.ratio)
        .ok_or_else(|| ProviderError::Infrastructure(format!("newapi group ratio is missing for group: {group}")))
}

pub(super) fn normalize_newapi_key(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("sk-") {
        trimmed.to_owned()
    } else {
        format!("sk-{trimmed}")
    }
}

pub(super) fn newapi_url(base_url: &str, path: &str) -> ProviderResult<String> {
    let base = base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err(ProviderError::InvalidInput("base_url cannot be blank".into()));
    }
    Ok(format!("{base}{path}"))
}

pub(super) async fn response_text(response: req::Response) -> ProviderResult<String> {
    let status = response.status();
    let text = req::response_text(response).await.map_err(client_error)?;
    if status.is_success() {
        return Ok(text);
    }
    Err(ProviderError::Infrastructure(format!("newapi returned {status}: {}", clipped_text(&text))))
}

pub(super) fn client_error(error: req::ClientError) -> ProviderError {
    ProviderError::Infrastructure(error.to_string())
}

pub(super) fn url_error(error: impl std::fmt::Display) -> ProviderError {
    ProviderError::InvalidInput(format!("invalid newapi url: {error}"))
}

fn clipped_text(value: &str) -> String {
    let clipped = value.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    if clipped.is_empty() { "(empty)".into() } else { clipped }
}

fn json_error(error: serde_json::Error) -> ProviderError {
    ProviderError::Infrastructure(format!("invalid newapi payload: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_newapi_key_adds_sk_prefix() {
        assert_eq!(normalize_newapi_key("abc"), "sk-abc");
        assert_eq!(normalize_newapi_key("sk-abc"), "sk-abc");
    }

    #[test]
    fn token_group_ratio_requires_known_group() {
        let groups = BTreeMap::from([("plus".into(), NewApiGroup { ratio: Decimal::new(3, 0) })]);
        assert_eq!(token_group_ratio(&groups, Some("plus")).unwrap(), Decimal::new(3, 0));
        assert!(token_group_ratio(&groups, Some("missing")).is_err());
    }

    #[test]
    fn decode_newapi_token_list_response() {
        let payload = r#"{"data":{"total":1,"items":[{"id":1209,"key":"c7mE**********9pAG","status":1,"name":"codex","group":"plus"}]},"success":true}"#;

        let envelope: TokenListEnvelope = decode_envelope(payload).unwrap();

        assert_eq!(envelope.data.total, 1);
        assert_eq!(envelope.data.items[0].id, 1209);
        assert_eq!(envelope.data.items[0].name, "codex");
    }

    #[test]
    fn decode_newapi_models_response() {
        let payload = r#"{"data":[{"id":"gpt-5.2","supported_endpoint_types":["openai"]}],"object":"list","success":true}"#;

        let envelope: ModelsEnvelope = decode_envelope(payload).unwrap();
        let models = envelope.data.into_iter().map(model_response).collect::<Vec<_>>();

        assert_eq!(models[0].id, "gpt-5.2");
        assert_eq!(models[0].supported_endpoint_types, vec!["openai"]);
    }
}
