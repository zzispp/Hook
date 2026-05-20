use axum::extract::Multipart;
use serde_json::{Map, Value};

use super::image::{DEFAULT_GPT_IMAGE_EDIT_QUALITY, normalize_count_text, normalize_quality, normalize_size};
use crate::llm_proxy::LlmProxyError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MultipartImageRequest {
    fields: Vec<MultipartField>,
    body: Value,
    model: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MultipartField {
    pub(super) name: String,
    pub(super) value: MultipartFieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MultipartFieldValue {
    Text(String),
    File {
        bytes: Vec<u8>,
        filename: Option<String>,
        content_type: Option<String>,
    },
}

impl MultipartImageRequest {
    pub(super) async fn from_multipart(mut multipart: Multipart) -> Result<Self, LlmProxyError> {
        let mut fields = Vec::new();
        while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
            let name = field
                .name()
                .map(str::to_owned)
                .ok_or_else(|| LlmProxyError::InvalidRequest("image edit multipart field name is required".into()))?;
            let filename = field.file_name().map(str::to_owned);
            let content_type = field.content_type().map(ToString::to_string);
            let bytes = field.bytes().await.map_err(multipart_error)?.to_vec();
            fields.push(MultipartField {
                name,
                value: multipart_field_value(bytes, filename, content_type)?,
            });
        }
        Self::from_fields(fields)
    }

    pub(super) fn from_fields(mut fields: Vec<MultipartField>) -> Result<Self, LlmProxyError> {
        normalize_multipart_fields(&mut fields)?;
        let body = multipart_body(&fields);
        let model = required_text_field(&fields, "model")?.to_owned();
        Ok(Self { fields, body, model })
    }

    pub(super) fn build_form(&self, provider_model_name: &str) -> Result<req::multipart::Form, LlmProxyError> {
        self.fields
            .iter()
            .try_fold(req::multipart::Form::new().text("model", provider_model_name.to_owned()), |form, field| {
                if field.name == "model" {
                    return Ok(form);
                }
                append_form_field(form, field)
            })
    }

    pub(super) fn model(&self) -> &str {
        &self.model
    }

    pub(super) fn provider_body(&self, provider_model_name: &str) -> Value {
        let mut body = self.body.clone();
        if let Some(object) = body.as_object_mut() {
            object.insert("model".into(), Value::String(provider_model_name.to_owned()));
        }
        body
    }

    pub(super) fn record_body(&self) -> &Value {
        &self.body
    }
}

fn normalize_multipart_fields(fields: &mut Vec<MultipartField>) -> Result<(), LlmProxyError> {
    let model = required_text_field(fields, "model")?.to_owned();
    required_text_field(fields, "prompt")?;
    ensure_image_file(fields)?;
    upsert_text_field(fields, "n", normalize_count_text(optional_text_field(fields, "n"))?.to_string());
    set_optional_multipart_text(fields, "size", normalize_size(&model, optional_text_field(fields, "size"))?);
    set_optional_multipart_text(
        fields,
        "quality",
        normalize_quality(&model, optional_text_field(fields, "quality"), DEFAULT_GPT_IMAGE_EDIT_QUALITY),
    );
    Ok(())
}

fn multipart_body(fields: &[MultipartField]) -> Value {
    let mut object = Map::new();
    for field in fields {
        merge_body_value(&mut object, &field.name, field.body_value());
    }
    Value::Object(object)
}

fn merge_body_value(object: &mut Map<String, Value>, key: &str, value: Value) {
    match object.remove(key) {
        None => {
            object.insert(key.to_owned(), value);
        }
        Some(Value::Array(mut items)) => {
            items.push(value);
            object.insert(key.to_owned(), Value::Array(items));
        }
        Some(existing) => {
            object.insert(key.to_owned(), Value::Array(vec![existing, value]));
        }
    }
}

impl MultipartField {
    fn body_value(&self) -> Value {
        match &self.value {
            MultipartFieldValue::Text(text) => Value::String(text.clone()),
            MultipartFieldValue::File { bytes, filename, content_type } => serde_json::json!({
                "filename": filename,
                "content_type": content_type,
                "size_bytes": bytes.len(),
            }),
        }
    }
}

fn multipart_field_value(bytes: Vec<u8>, filename: Option<String>, content_type: Option<String>) -> Result<MultipartFieldValue, LlmProxyError> {
    match filename {
        Some(filename) => Ok(MultipartFieldValue::File {
            bytes,
            filename: Some(filename),
            content_type,
        }),
        None => String::from_utf8(bytes)
            .map(MultipartFieldValue::Text)
            .map_err(|error| LlmProxyError::InvalidRequest(format!("image edit multipart text field must be valid UTF-8: {error}"))),
    }
}

fn append_form_field(form: req::multipart::Form, field: &MultipartField) -> Result<req::multipart::Form, LlmProxyError> {
    match &field.value {
        MultipartFieldValue::Text(text) => Ok(form.text(field.name.clone(), text.clone())),
        MultipartFieldValue::File { bytes, filename, content_type } => {
            let mut part = req::multipart::Part::bytes(bytes.clone());
            if let Some(filename) = filename {
                part = part.file_name(filename.clone());
            }
            if let Some(content_type) = content_type {
                part = part
                    .mime_str(content_type)
                    .map_err(|error| LlmProxyError::InvalidRequest(format!("invalid multipart content type {content_type:?}: {error}")))?;
            }
            Ok(form.part(field.name.clone(), part))
        }
    }
}

fn required_text_field<'a>(fields: &'a [MultipartField], key: &str) -> Result<&'a str, LlmProxyError> {
    optional_text_field(fields, key)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| LlmProxyError::InvalidRequest(format!("{key} is required")))
}

fn optional_text_field<'a>(fields: &'a [MultipartField], key: &str) -> Option<&'a str> {
    fields.iter().find_map(|field| match (&field.name[..], &field.value) {
        (name, MultipartFieldValue::Text(value)) if name == key => Some(value.as_str()),
        _ => None,
    })
}

fn ensure_image_file(fields: &[MultipartField]) -> Result<(), LlmProxyError> {
    fields
        .iter()
        .any(|field| is_image_field(&field.name) && matches!(field.value, MultipartFieldValue::File { .. }))
        .then_some(())
        .ok_or_else(|| LlmProxyError::InvalidRequest("image is required".into()))
}

fn is_image_field(name: &str) -> bool {
    name == "image" || name == "image[]" || name.starts_with("image[")
}

fn set_optional_multipart_text(fields: &mut Vec<MultipartField>, key: &str, value: Option<String>) {
    match value {
        Some(value) => upsert_text_field(fields, key, value),
        None => remove_text_fields(fields, key),
    }
}

fn upsert_text_field(fields: &mut Vec<MultipartField>, key: &str, value: String) {
    if let Some(field) = fields
        .iter_mut()
        .find(|field| field.name == key && matches!(field.value, MultipartFieldValue::Text(_)))
    {
        field.value = MultipartFieldValue::Text(value);
        return;
    }
    fields.push(MultipartField {
        name: key.to_owned(),
        value: MultipartFieldValue::Text(value),
    });
}

fn remove_text_fields(fields: &mut Vec<MultipartField>, key: &str) {
    fields.retain(|field| !(field.name == key && matches!(field.value, MultipartFieldValue::Text(_))));
}

fn multipart_error(error: impl std::fmt::Display) -> LlmProxyError {
    LlmProxyError::InvalidRequest(format!("invalid image edit multipart body: {error}"))
}
