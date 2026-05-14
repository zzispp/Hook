use std::io::{Read, Write};

use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use serde_json::{Value, json};

use crate::{StorageError, StorageResult};

const COMPRESSED_MARKER_KEY: &str = "__hook_compressed_payload__";
const COMPRESSED_ENCODING: &str = "zlib+hex";
const COMPRESSED_DATA_KEY: &str = "data";
const COMPRESSED_SIZE_KEY: &str = "original_size";

pub(super) fn decode_payload(value: Option<String>) -> StorageResult<Option<Value>> {
    value.map(|text| decode_payload_text(&text)).transpose()
}

pub(super) fn compress_payload(value: Option<String>) -> StorageResult<Option<String>> {
    let Some(text) = value else {
        return Ok(None);
    };
    if is_compressed_text(&text)? {
        return Ok(Some(text));
    }
    let compressed = compress_text(&text)?;
    let wrapper = json!({
        COMPRESSED_MARKER_KEY: true,
        "encoding": COMPRESSED_ENCODING,
        COMPRESSED_SIZE_KEY: text.len(),
        COMPRESSED_DATA_KEY: hex::encode(compressed),
    });
    serde_json::to_string(&wrapper)
        .map(Some)
        .map_err(|error| StorageError::Database(error.to_string()))
}

fn decode_payload_text(text: &str) -> StorageResult<Value> {
    let value: Value = serde_json::from_str(text).map_err(|error| StorageError::Database(error.to_string()))?;
    if !is_compressed_value(&value) {
        return Ok(value);
    }
    let encoded = wrapper_text(&value, COMPRESSED_DATA_KEY)?;
    let bytes = hex::decode(encoded).map_err(|error| StorageError::Database(error.to_string()))?;
    let inflated = decompress_bytes(&bytes)?;
    serde_json::from_slice(&inflated).map_err(|error| StorageError::Database(error.to_string()))
}

fn is_compressed_text(text: &str) -> StorageResult<bool> {
    let value: Value = serde_json::from_str(text).map_err(|error| StorageError::Database(error.to_string()))?;
    Ok(is_compressed_value(&value))
}

fn is_compressed_value(value: &Value) -> bool {
    let Value::Object(map) = value else {
        return false;
    };
    map.get(COMPRESSED_MARKER_KEY).and_then(Value::as_bool) == Some(true)
}

fn wrapper_text<'a>(value: &'a Value, key: &str) -> StorageResult<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|item| !item.is_empty())
        .ok_or_else(|| StorageError::Database(format!("compressed payload missing {key}")))
}

fn compress_text(text: &str) -> StorageResult<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(text.as_bytes()).map_err(|error| StorageError::Database(error.to_string()))?;
    encoder.finish().map_err(|error| StorageError::Database(error.to_string()))
}

fn decompress_bytes(bytes: &[u8]) -> StorageResult<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).map_err(|error| StorageError::Database(error.to_string()))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{compress_payload, decode_payload};

    #[test]
    fn decode_payload_reads_plain_json() {
        let payload = Some(r#"{"model":"gpt-5.5"}"#.to_owned());
        assert_eq!(decode_payload(payload).unwrap(), Some(json!({"model": "gpt-5.5"})));
    }

    #[test]
    fn compress_payload_round_trips() {
        let original = Some(r#"{"error":{"message":"quota exceeded","code":"insufficient_quota"}}"#.to_owned());
        let compressed = compress_payload(original).unwrap();
        let restored = decode_payload(compressed).unwrap();
        assert_eq!(restored, Some(json!({"error": {"message": "quota exceeded", "code": "insufficient_quota"}})));
    }

    #[test]
    fn compress_payload_is_idempotent_for_wrapped_value() {
        let original = Some(r#"{"model":"gpt-5.5"}"#.to_owned());
        let compressed = compress_payload(original).unwrap();
        assert_eq!(compress_payload(compressed.clone()).unwrap(), compressed);
    }
}
