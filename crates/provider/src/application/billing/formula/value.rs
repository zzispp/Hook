use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde_json::Value;

use crate::application::billing::types::quantize;

pub(super) fn value_decimal(value: &Value) -> Result<Decimal, String> {
    match value {
        Value::Number(number) => Decimal::from_f64(number.as_f64().ok_or_else(|| "invalid number".to_owned())?).ok_or_else(|| "invalid decimal".into()),
        Value::String(text) => text.parse::<Decimal>().map_err(|error| error.to_string()),
        _ => Err("value is not numeric".into()),
    }
}

pub(super) fn decimal_json(value: Decimal) -> Value {
    Value::String(quantize(value).to_string())
}
