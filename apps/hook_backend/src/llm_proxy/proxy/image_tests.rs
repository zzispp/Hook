use serde_json::json;

use super::super::image_form::{MultipartField, MultipartFieldValue, MultipartImageRequest};
use super::validate_image_json;

#[test]
fn validate_image_json_applies_newapi_defaults() {
    let body = validate_image_json(json!({
        "model": "dall-e-3",
        "prompt": "a lantern on a pier"
    }))
    .unwrap();

    assert_eq!(body["n"], 1);
    assert_eq!(body["size"], "1024x1024");
    assert_eq!(body["quality"], "standard");

    let gpt_image = validate_image_json(json!({
        "model": "gpt-image-1",
        "prompt": "a lantern on a pier"
    }))
    .unwrap();

    assert_eq!(gpt_image["quality"], "auto");
}

#[test]
fn validate_image_json_rejects_invalid_size_separator() {
    let error = validate_image_json(json!({
        "model": "dall-e-3",
        "prompt": "x",
        "size": "1024×1024"
    }))
    .unwrap_err();

    assert!(matches!(error, crate::llm_proxy::LlmProxyError::InvalidRequest(message) if message.contains("please use 'x' instead of the multiplication sign")));
}

#[test]
fn multipart_image_request_extracts_image_and_mask_parts() {
    let request = MultipartImageRequest::from_fields(vec![
        text("model", "gpt-image-1"),
        text("prompt", "restore the photo"),
        file("image", "input.png", "image/png", b"png-bytes"),
        file("mask", "mask.png", "image/png", b"mask-bytes"),
    ])
    .unwrap();

    assert_eq!(request.model(), "gpt-image-1");
    assert_eq!(request.record_body()["quality"], "standard");
    assert_eq!(request.record_body()["n"], "1");
    assert_eq!(request.record_body()["image"]["filename"], "input.png");
    assert_eq!(request.record_body()["mask"]["filename"], "mask.png");
}

#[test]
fn multipart_image_request_detects_stream_true() {
    let request = MultipartImageRequest::from_fields(vec![
        text("model", "gpt-image-1"),
        text("prompt", "restore the photo"),
        text("stream", "true"),
        file("image", "input.png", "image/png", b"png-bytes"),
    ])
    .unwrap();

    assert!(request.is_stream());
}

#[test]
fn multipart_image_request_requires_image_file() {
    let error = MultipartImageRequest::from_fields(vec![text("model", "gpt-image-1"), text("prompt", "restore the photo")]).unwrap_err();

    assert!(matches!(error, crate::llm_proxy::LlmProxyError::InvalidRequest(message) if message == "image is required"));
}

fn text(name: &str, value: &str) -> MultipartField {
    MultipartField {
        name: name.into(),
        value: MultipartFieldValue::Text(value.into()),
    }
}

fn file(name: &str, filename: &str, content_type: &str, bytes: &[u8]) -> MultipartField {
    MultipartField {
        name: name.into(),
        value: MultipartFieldValue::File {
            bytes: bytes.to_vec(),
            filename: Some(filename.into()),
            content_type: Some(content_type.into()),
        },
    }
}
