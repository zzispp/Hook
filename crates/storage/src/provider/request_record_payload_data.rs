use std::io::{Read, Write};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use sha2::{Digest, Sha256};

use crate::{StorageError, StorageResult};

use super::request_record_payload_store::RequestPayloadData;

pub(super) fn compress_json(value: &serde_json::Value) -> StorageResult<RequestPayloadData> {
    let raw = serde_json::to_vec(value).map_err(|error| StorageError::Database(format!("encode request payload failed: {error}")))?;
    let compressed = gzip_bytes(&raw)?;
    Ok(RequestPayloadData {
        original_size: checked_len(raw.len(), "request payload")?,
        compressed_size: checked_len(compressed.len(), "compressed request payload")?,
        sha256: hex::encode(Sha256::digest(&raw)),
        compressed_payload: compressed,
    })
}

pub(super) fn decode_json(bytes: &[u8]) -> StorageResult<serde_json::Value> {
    let decoded = gunzip_bytes(bytes)?;
    serde_json::from_slice(&decoded).map_err(|error| StorageError::Database(format!("decode request payload JSON failed: {error}")))
}

pub(super) fn json_size(value: &serde_json::Value) -> StorageResult<i64> {
    let bytes = serde_json::to_vec(value).map_err(|error| StorageError::Database(format!("encode request payload failed: {error}")))?;
    checked_len(bytes.len(), "request payload")
}

fn gzip_bytes(bytes: &[u8]) -> StorageResult<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|error| StorageError::Database(format!("gzip request payload failed: {error}")))?;
    encoder
        .finish()
        .map_err(|error| StorageError::Database(format!("finish gzip request payload failed: {error}")))
}

fn gunzip_bytes(bytes: &[u8]) -> StorageResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|error| StorageError::Database(format!("gunzip request payload failed: {error}")))?;
    Ok(decoded)
}

fn checked_len(value: usize, label: &str) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::Database(format!("{label} size exceeds PostgreSQL bigint range")))
}
