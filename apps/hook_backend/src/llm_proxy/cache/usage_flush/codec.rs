use std::collections::HashMap;

use rust_decimal::Decimal;
use storage::{
    api_token::ApiTokenUsageRecord,
    model::{GlobalModelUsageRecord, GlobalModelUserUsageRecord},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::llm_proxy::LlmProxyError;

const TOKEN_COST_SCALE: u32 = 8;

pub(super) fn decode_token_usage_batch(
    mut cost: HashMap<String, String>,
    mut count: HashMap<String, String>,
    mut last_used_at: HashMap<String, String>,
) -> Result<Vec<ApiTokenUsageRecord>, LlmProxyError> {
    if cost.is_empty() && count.is_empty() && last_used_at.is_empty() {
        return Ok(Vec::new());
    }
    let mut records = Vec::with_capacity(cost.len());
    for (token_id, cost_units) in cost.drain() {
        let request_count = count
            .remove(&token_id)
            .ok_or_else(|| decode_error("token request_count missing for pending usage"))?;
        let used_at = last_used_at
            .remove(&token_id)
            .ok_or_else(|| decode_error("token last_used_at missing for pending usage"))?;
        records.push(ApiTokenUsageRecord {
            token_id,
            cost: Decimal::new(parse_i64(&cost_units, "token cost units")?, TOKEN_COST_SCALE),
            request_count: parse_i64(&request_count, "token request_count")?,
            used_at: OffsetDateTime::parse(&used_at, &Rfc3339).map_err(|error| decode_error(&format!("invalid token last_used_at: {error}")))?,
        });
    }
    if !count.is_empty() || !last_used_at.is_empty() {
        return Err(decode_error("token usage pending hashes are out of sync"));
    }
    Ok(records)
}

pub(super) fn decode_model_usage_batch(mut counts: HashMap<String, String>) -> Result<Vec<GlobalModelUsageRecord>, LlmProxyError> {
    let mut records = Vec::with_capacity(counts.len());
    for (model_id, count) in counts.drain() {
        records.push(GlobalModelUsageRecord {
            model_id,
            count: parse_i64(&count, "model usage count")?,
            user_id: None,
        });
    }
    Ok(records)
}

pub(super) fn decode_user_model_usage_batch(mut counts: HashMap<String, String>) -> Result<Vec<GlobalModelUserUsageRecord>, LlmProxyError> {
    let mut records = Vec::with_capacity(counts.len());
    for (key, count) in counts.drain() {
        let (user_id, model_id) = decode_user_model_key(&key)?;
        records.push(GlobalModelUserUsageRecord {
            user_id,
            model_id,
            count: parse_i64(&count, "user model usage count")?,
        });
    }
    Ok(records)
}

pub(super) fn encode_user_model_key(user_id: &str, model_id: &str) -> String {
    format!("{}:{user_id}{model_id}", user_id.len())
}

fn decode_user_model_key(value: &str) -> Result<(String, String), LlmProxyError> {
    let Some((length, rest)) = value.split_once(':') else {
        return Err(decode_error("invalid user model usage key"));
    };
    let length = length
        .parse::<usize>()
        .map_err(|error| decode_error(&format!("invalid user model usage key length: {error}")))?;
    if rest.len() < length {
        return Err(decode_error("invalid user model usage key payload"));
    }
    let (user_id, model_id) = rest.split_at(length);
    if user_id.is_empty() || model_id.is_empty() {
        return Err(decode_error("invalid user model usage key parts"));
    }
    Ok((user_id.to_owned(), model_id.to_owned()))
}

pub(super) fn token_cost_units(cost: Decimal) -> Result<i64, LlmProxyError> {
    let normalized = cost.round_dp(TOKEN_COST_SCALE);
    if normalized != cost {
        return Err(LlmProxyError::Infrastructure(format!(
            "token usage cost exceeds {TOKEN_COST_SCALE} decimal places: {cost}"
        )));
    }
    let mut scaled = normalized;
    scaled.rescale(TOKEN_COST_SCALE);
    i64::try_from(scaled.mantissa()).map_err(|_| LlmProxyError::Infrastructure(format!("token usage cost overflowed fixed-point range: {cost}")))
}

fn parse_i64(value: &str, label: &str) -> Result<i64, LlmProxyError> {
    value.parse::<i64>().map_err(|error| decode_error(&format!("invalid {label}: {error}")))
}

fn decode_error(message: &str) -> LlmProxyError {
    LlmProxyError::Infrastructure(message.into())
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use rust_decimal::Decimal;

    use super::{decode_token_usage_batch, decode_user_model_usage_batch, encode_user_model_key, token_cost_units};

    #[test]
    fn token_cost_units_preserve_eight_decimal_scale() {
        let cost = Decimal::from_str("0.03127500").unwrap();

        assert_eq!(token_cost_units(cost).unwrap(), 3_127_500);
    }

    #[test]
    fn token_cost_units_reject_more_than_eight_decimals() {
        let error = token_cost_units(Decimal::from_str("0.000000001").unwrap()).unwrap_err();

        assert!(error.to_string().contains("exceeds 8 decimal places"));
    }

    #[test]
    fn decode_token_usage_batch_rejects_out_of_sync_hashes() {
        let cost = HashMap::from([("token-1".into(), "100".into())]);
        let count = HashMap::new();
        let last_used_at = HashMap::from([("token-1".into(), "2026-05-15T10:00:00Z".into())]);

        let error = decode_token_usage_batch(cost, count, last_used_at).unwrap_err();

        assert!(error.to_string().contains("request_count missing"));
    }

    #[test]
    fn user_model_usage_key_round_trips() {
        let key = encode_user_model_key("user:1", "model:2");
        let records = decode_user_model_usage_batch(HashMap::from([(key, "3".into())])).unwrap();

        assert_eq!(records[0].user_id, "user:1");
        assert_eq!(records[0].model_id, "model:2");
        assert_eq!(records[0].count, 3);
    }
}
