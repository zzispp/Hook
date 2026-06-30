use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use crate::application::{ProviderError, ProviderResult, UpstreamImportModel, UpstreamSyncToken};

const ACTIVE_STATUS: &str = "active";
const MAX_ERROR_BODY_CHARS: usize = 300;

#[derive(Debug, Deserialize)]
pub(super) struct Sub2ApiEnvelope<T> {
    pub(super) code: i32,
    pub(super) message: String,
    pub(super) data: T,
}

#[derive(Debug, Deserialize)]
struct RawTokenRefreshResponse {
    access_token: String,
    refresh_token: String,
    token_expires_at: Option<String>,
}

#[derive(Debug)]
pub(super) struct TokenRefreshResponse {
    pub(super) access_token: String,
    pub(super) refresh_token: String,
    pub(super) token_expires_at: String,
}

impl<'de> Deserialize<'de> for TokenRefreshResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawTokenRefreshResponse::deserialize(deserializer)?;
        let token_expires_at = match raw.token_expires_at {
            Some(value) if !value.trim().is_empty() => value,
            _ => token_expiration_from_jwt(&raw.access_token).map_err(serde::de::Error::custom)?,
        };
        Ok(Self {
            access_token: raw.access_token,
            refresh_token: raw.refresh_token,
            token_expires_at,
        })
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct PaginatedKeys {
    pub(super) items: Vec<Sub2ApiKeyRecord>,
    pub(super) total: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct Sub2ApiKeyRecord {
    pub(super) id: i64,
    pub(super) key: String,
    pub(super) name: String,
    pub(super) status: String,
    pub(super) quota: f64,
    pub(super) quota_used: f64,
    pub(super) expires_at: Option<String>,
    pub(super) group: Option<Sub2ApiGroupRecord>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct Sub2ApiGroupRecord {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) rate_multiplier: f64,
    pub(super) status: String,
}

impl Sub2ApiGroupRecord {
    pub(super) fn is_active(&self) -> bool {
        self.status == ACTIVE_STATUS
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct Sub2ApiModelEnvelope {
    pub(super) data: Vec<Sub2ApiModelRecord>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Sub2ApiModelRecord {
    pub(super) id: String,
}

pub(super) type UserGroupRates = BTreeMap<String, f64>;

pub(super) fn decode_envelope<T>(text: &str) -> ProviderResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let envelope: Sub2ApiEnvelope<T> = serde_json::from_str(text).map_err(json_error)?;
    if envelope.code == 0 {
        return Ok(envelope.data);
    }
    Err(ProviderError::Infrastructure(format!("sub2api returned failure: {}", envelope.message)))
}

pub(super) fn decode_models(text: &str) -> ProviderResult<Vec<UpstreamImportModel>> {
    let envelope: Sub2ApiModelEnvelope = serde_json::from_str(text).map_err(json_error)?;
    Ok(envelope
        .data
        .into_iter()
        .map(|model| UpstreamImportModel {
            id: model.id,
            supported_endpoint_types: vec!["openai".into()],
        })
        .collect())
}

pub(super) fn sub2api_url(base_url: &str, path: &str) -> ProviderResult<String> {
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
    Err(ProviderError::Infrastructure(format!("sub2api returned {status}: {}", clipped_text(&text))))
}

pub(super) fn key_status(record: &Sub2ApiKeyRecord) -> ProviderResult<String> {
    if record.status == ACTIVE_STATUS && quota_exhausted(record) {
        return Ok("quota_exhausted".into());
    }
    if record.status == ACTIVE_STATUS && expired(record)? {
        return Ok("expired".into());
    }
    Ok(record.status.clone())
}

pub(super) fn key_is_active(record: &Sub2ApiKeyRecord) -> ProviderResult<bool> {
    Ok(key_status(record)? == ACTIVE_STATUS)
}

pub(super) fn sync_token(record: Sub2ApiKeyRecord) -> ProviderResult<UpstreamSyncToken> {
    let status = key_status(&record)?;
    Ok(UpstreamSyncToken {
        id: record.id.to_string(),
        name: record.name,
        masked_key: masked_key(&record.key),
        is_active: status == ACTIVE_STATUS,
        status,
        group_id: record.group.as_ref().map(|group| group.id.to_string()),
        group: record.group.as_ref().map(|group| group.name.clone()),
    })
}

pub(super) fn group_ratio(record: &Sub2ApiKeyRecord, user_group_rates: &UserGroupRates) -> ProviderResult<Option<Decimal>> {
    let Some(group) = record.group.as_ref() else {
        return Ok(None);
    };
    if !group.is_active() {
        return Err(ProviderError::Infrastructure(format!("sub2api group is inactive: {}", group.name)));
    }
    let user_ratio = user_group_rates.get(&group.id.to_string()).copied();
    decimal_from_f64(user_ratio.unwrap_or(group.rate_multiplier))
        .map(Some)
        .map_err(|error| ProviderError::Infrastructure(format!("invalid sub2api group ratio for {}: {error}", group.name)))
}

pub(super) fn masked_key(value: &str) -> String {
    let chars = value.trim().chars().collect::<Vec<_>>();
    if chars.is_empty() {
        return String::new();
    }
    if chars.len() <= 8 {
        let head = chars.iter().take(2).collect::<String>();
        let tail = chars.iter().rev().take(2).collect::<Vec<_>>().into_iter().rev().collect::<String>();
        return format!("{head}****{tail}");
    }
    let head = chars.iter().take(4).collect::<String>();
    let tail = chars.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<String>();
    format!("{head}****{tail}")
}

pub(super) fn client_error(error: req::ClientError) -> ProviderError {
    ProviderError::Infrastructure(error.to_string())
}

pub(super) fn url_error(error: impl std::fmt::Display) -> ProviderError {
    ProviderError::InvalidInput(format!("invalid sub2api url: {error}"))
}

fn clipped_text(value: &str) -> String {
    let clipped = value.chars().take(MAX_ERROR_BODY_CHARS).collect::<String>();
    if clipped.is_empty() { "(empty)".into() } else { clipped }
}

fn json_error(error: serde_json::Error) -> ProviderError {
    ProviderError::Infrastructure(format!("invalid sub2api payload: {error}"))
}

fn token_expiration_from_jwt(token: &str) -> Result<String, String> {
    let payload = token
        .trim()
        .split('.')
        .nth(1)
        .ok_or_else(|| "sub2api token_expires_at is missing and access_token is not a JWT".to_owned())?;
    let decoded = decode_base64url(payload)?;
    let value: Value =
        serde_json::from_slice(&decoded).map_err(|error| format!("sub2api token_expires_at is missing and access_token payload is invalid: {error}"))?;
    let exp = value
        .get("exp")
        .and_then(Value::as_i64)
        .ok_or_else(|| "sub2api token_expires_at is missing and access_token payload has no exp".to_owned())?;
    Ok((i128::from(exp) * 1000).to_string())
}

fn decode_base64url(value: &str) -> Result<Vec<u8>, String> {
    let normalized = value.replace('-', "+").replace('_', "/");
    let padding = (4 - normalized.len() % 4) % 4;
    let padded = format!("{normalized}{}", "=".repeat(padding));
    base64_decode(&padded)
}

fn base64_decode(value: &str) -> Result<Vec<u8>, String> {
    const INVALID: u8 = u8::MAX;
    let mut table = [INVALID; 256];
    for (index, byte) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
        table[*byte as usize] = index as u8;
    }
    let bytes = value.as_bytes();
    if !bytes.len().is_multiple_of(4) {
        return Err("invalid base64 length".into());
    }
    let mut output = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let mut values = [0_u8; 4];
        let mut padding = 0;
        for (index, byte) in chunk.iter().enumerate() {
            if *byte == b'=' {
                values[index] = 0;
                padding += 1;
                continue;
            }
            let decoded = table[*byte as usize];
            if decoded == INVALID {
                return Err(format!("invalid base64 character: {}", *byte as char));
            }
            values[index] = decoded;
        }
        output.push((values[0] << 2) | (values[1] >> 4));
        if padding < 2 {
            output.push((values[1] << 4) | (values[2] >> 2));
        }
        if padding < 1 {
            output.push((values[2] << 6) | values[3]);
        }
    }
    Ok(output)
}

fn quota_exhausted(record: &Sub2ApiKeyRecord) -> bool {
    record.quota > 0.0 && record.quota_used >= record.quota
}

fn expired(record: &Sub2ApiKeyRecord) -> ProviderResult<bool> {
    let Some(expires_at) = record.expires_at.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(false);
    };
    let expires_at =
        time::OffsetDateTime::parse(expires_at, &Rfc3339).map_err(|error| ProviderError::Infrastructure(format!("invalid sub2api expires_at: {error}")))?;
    Ok(expires_at <= time::OffsetDateTime::now_utc())
}

fn decimal_from_f64(value: f64) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str_exact(&value.to_string())
}
