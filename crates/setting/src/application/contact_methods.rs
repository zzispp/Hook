use types::system_setting::{ContactMethod, ContactMethodType, public_base_url_is_valid};

use super::{SettingError, SettingResult};

const MAX_CONTACT_FIELD_LENGTH: usize = 255;
const MAX_CONTACT_QR_CODE_URL_LENGTH: usize = 4096;
const MAX_CONTACT_QR_CODE_DATA_URL_LENGTH: usize = 10_485_760; // 10M

pub(super) fn sanitize_contact_methods(methods: Vec<ContactMethod>) -> Vec<ContactMethod> {
    methods
        .into_iter()
        .map(|method| ContactMethod {
            id: method.id.trim().to_owned(),
            method_type: method.method_type,
            custom_type: method.custom_type.trim().to_owned(),
            icon: method.icon.trim().to_owned(),
            value: method.value.trim().to_owned(),
            qr_code: method.qr_code.trim().to_owned(),
        })
        .collect()
}

pub(super) fn validate_contact_methods(methods: Option<&[ContactMethod]>) -> SettingResult<()> {
    let Some(methods) = methods else {
        return Ok(());
    };
    for method in methods {
        validate_contact_method(method)?;
    }
    Ok(())
}

fn validate_contact_method(method: &ContactMethod) -> SettingResult<()> {
    validate_required_length("contact_methods.id", &method.id, MAX_CONTACT_FIELD_LENGTH)?;
    validate_required_length("contact_methods.icon", &method.icon, MAX_CONTACT_FIELD_LENGTH)?;
    validate_required_length("contact_methods.value", &method.value, MAX_CONTACT_FIELD_LENGTH)?;
    validate_custom_contact_type(method)?;
    validate_contact_qr_code(&method.qr_code)
}

fn validate_custom_contact_type(method: &ContactMethod) -> SettingResult<()> {
    if method.method_type != ContactMethodType::Custom {
        return Ok(());
    }
    validate_required_length("contact_methods.custom_type", &method.custom_type, MAX_CONTACT_FIELD_LENGTH)
}

fn validate_contact_qr_code(value: &str) -> SettingResult<()> {
    if value.is_empty() {
        return Ok(());
    }
    if value.starts_with("data:image/") {
        return validate_qr_code_length(value, MAX_CONTACT_QR_CODE_DATA_URL_LENGTH);
    }
    if public_url_is_valid(value)? {
        return validate_qr_code_length(value, MAX_CONTACT_QR_CODE_URL_LENGTH);
    }
    Err(SettingError::InvalidInput(
        "contact_methods.qr_code must be an image data URL or a valid HTTP/HTTPS URL".into(),
    ))
}

fn validate_qr_code_length(value: &str, max: usize) -> SettingResult<()> {
    if value.len() > max {
        return Err(SettingError::InvalidInput(format!("contact_methods.qr_code length must be at most {max}")));
    }
    Ok(())
}

fn public_url_is_valid(value: &str) -> SettingResult<bool> {
    public_base_url_is_valid(value).map_err(|error| SettingError::Infrastructure(format!("invalid URL validation regex: {error}")))
}

fn validate_required_length(field: &str, value: &str, max: usize) -> SettingResult<()> {
    if value.is_empty() || value.len() > max {
        return Err(SettingError::InvalidInput(format!("{field} length must be between 1 and {max}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use types::system_setting::{ContactMethod, ContactMethodType};

    use super::{sanitize_contact_methods, validate_contact_methods};

    #[test]
    fn sanitize_contact_methods_trims_text_fields() {
        let sanitized = sanitize_contact_methods(vec![contact_method("  wechat-1  ", ContactMethodType::Wechat, "  wx  ")]);
        let method = &sanitized[0];

        assert_eq!(method.id, "wechat-1");
        assert_eq!(method.value, "wx");
    }

    #[test]
    fn validate_contact_methods_accepts_qr_code_data_url() {
        let methods = vec![ContactMethod {
            qr_code: "data:image/png;base64,AA==".into(),
            ..contact_method("wechat", ContactMethodType::Wechat, "hook")
        }];

        assert!(validate_contact_methods(Some(&methods)).is_ok());
    }

    #[test]
    fn validate_contact_methods_accepts_uploaded_qr_code_data_url() {
        let methods = vec![ContactMethod {
            qr_code: format!("data:image/png;base64,{}", "A".repeat(8192)),
            ..contact_method("wechat", ContactMethodType::Wechat, "hook")
        }];

        assert!(validate_contact_methods(Some(&methods)).is_ok());
    }

    #[test]
    fn validate_contact_methods_accepts_qr_code_http_url() {
        let methods = vec![ContactMethod {
            qr_code: "https://example.com/qr.png".into(),
            ..contact_method("telegram", ContactMethodType::Telegram, "https://t.me/hook")
        }];

        assert!(validate_contact_methods(Some(&methods)).is_ok());
    }

    #[test]
    fn validate_contact_methods_accepts_custom_contact_type() {
        let methods = vec![ContactMethod {
            custom_type: "客服".into(),
            ..contact_method("custom", ContactMethodType::Custom, "support")
        }];

        assert!(validate_contact_methods(Some(&methods)).is_ok());
    }

    #[test]
    fn validate_contact_methods_rejects_missing_value() {
        let methods = vec![contact_method("wechat", ContactMethodType::Wechat, "")];
        let error = validate_contact_methods(Some(&methods)).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: contact_methods.value length must be between 1 and 255");
    }

    #[test]
    fn validate_contact_methods_rejects_custom_missing_type() {
        let methods = vec![contact_method("custom", ContactMethodType::Custom, "support")];
        let error = validate_contact_methods(Some(&methods)).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: contact_methods.custom_type length must be between 1 and 255");
    }

    #[test]
    fn validate_contact_methods_rejects_invalid_qr_code() {
        let methods = vec![ContactMethod {
            qr_code: "ftp://example.com/qr.png".into(),
            ..contact_method("qq", ContactMethodType::Qq, "12345")
        }];
        let error = validate_contact_methods(Some(&methods)).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: contact_methods.qr_code must be an image data URL or a valid HTTP/HTTPS URL"
        );
    }

    fn contact_method(id: &str, method_type: ContactMethodType, value: &str) -> ContactMethod {
        ContactMethod {
            id: id.into(),
            method_type,
            custom_type: String::new(),
            icon: "simple-icons:wechat".into(),
            value: value.into(),
            qr_code: String::new(),
        }
    }
}
