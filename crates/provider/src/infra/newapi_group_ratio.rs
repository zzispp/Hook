use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

use crate::application::{ProviderError, ProviderResult, UpstreamGroupRatio};

const AUTO_GROUP_RATIO: &str = "自动";

#[derive(Debug, Clone, PartialEq)]
pub(super) struct NewApiGroupRatio {
    value: UpstreamGroupRatio,
}

impl NewApiGroupRatio {
    pub(super) fn fixed_ratio(&self, group: &str) -> ProviderResult<Decimal> {
        match &self.value {
            UpstreamGroupRatio::Fixed(value) => Ok(*value),
            UpstreamGroupRatio::UpstreamValue(value) => Err(ProviderError::Infrastructure(format!(
                "newapi group ratio is not fixed for group {group}: {value}"
            ))),
        }
    }

    pub(super) fn is_fixed(&self) -> bool {
        matches!(&self.value, UpstreamGroupRatio::Fixed(_))
    }

    pub(super) fn into_value(self) -> UpstreamGroupRatio {
        self.value
    }

    #[cfg(test)]
    pub(super) fn fixed_for_test(value: Decimal) -> Self {
        Self {
            value: UpstreamGroupRatio::Fixed(value),
        }
    }
}

impl<'de> Deserialize<'de> for NewApiGroupRatio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        parse_ratio_value(value).map(|value| Self { value }).map_err(serde::de::Error::custom)
    }
}

fn parse_ratio_value(value: serde_json::Value) -> Result<UpstreamGroupRatio, String> {
    match value {
        serde_json::Value::Number(number) => decimal_from_number(number),
        serde_json::Value::String(value) => ratio_from_string(value),
        other => Err(format!("expected number or string for newapi group ratio, got {}", type_name(&other))),
    }
}

fn decimal_from_number(number: serde_json::Number) -> Result<UpstreamGroupRatio, String> {
    number
        .to_string()
        .parse::<Decimal>()
        .map(UpstreamGroupRatio::Fixed)
        .map_err(|error| format!("invalid newapi group ratio number {number}: {error}"))
}

fn ratio_from_string(value: String) -> Result<UpstreamGroupRatio, String> {
    let trimmed = value.trim();
    if trimmed == AUTO_GROUP_RATIO {
        return Ok(UpstreamGroupRatio::UpstreamValue(value));
    }
    trimmed
        .parse::<Decimal>()
        .map(UpstreamGroupRatio::Fixed)
        .map_err(|error| format!("invalid newapi group ratio string {value:?}: {error}"))
}

fn type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_number_ratio() {
        let ratio: NewApiGroupRatio = serde_json::from_str("2.5").unwrap();

        assert_eq!(ratio.fixed_ratio("plus").unwrap(), Decimal::new(25, 1));
    }

    #[test]
    fn parses_numeric_string_ratio() {
        let ratio: NewApiGroupRatio = serde_json::from_str(r#""2.5""#).unwrap();

        assert_eq!(ratio.fixed_ratio("plus").unwrap(), Decimal::new(25, 1));
    }

    #[test]
    fn preserves_auto_ratio_as_upstream_value() {
        let ratio: NewApiGroupRatio = serde_json::from_str(r#""自动""#).unwrap();

        assert_eq!(
            ratio.fixed_ratio("auto").unwrap_err().to_string(),
            "infrastructure error: newapi group ratio is not fixed for group auto: 自动"
        );
    }
}
