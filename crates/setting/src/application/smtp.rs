use types::system_setting::{SmtpEncryption, SystemSettingsSmtpTestRequest, SystemSettingsSmtpTestResponse};

use super::{SettingResult, SettingSecretCipher};

const SMTP_TEST_SUCCESS_MESSAGE: &str = "SMTP 连接测试成功";
const MISSING_SMTP_CONFIG_MESSAGE: &str = "SMTP 配置不完整，请检查 ";
const MAX_SMTP_HOST_LENGTH: usize = 255;
const MAX_SMTP_USERNAME_LENGTH: usize = 255;
const MAX_SMTP_PASSWORD_LENGTH: usize = 1024;
const MAX_SMTP_FROM_EMAIL_LENGTH: usize = 255;
const MAX_SMTP_FROM_NAME_LENGTH: usize = 100;
const MIN_SMTP_PORT: i64 = 1;
const MAX_SMTP_PORT: i64 = 65_535;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredSmtpSettings {
    pub smtp_host: String,
    pub smtp_port: i64,
    pub smtp_username: String,
    pub encrypted_smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_encryption: SmtpEncryption,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmtpConnectionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub encryption: SmtpEncryption,
}

pub fn success_response() -> SystemSettingsSmtpTestResponse {
    SystemSettingsSmtpTestResponse::succeeded(SMTP_TEST_SUCCESS_MESSAGE)
}

pub fn failure_response(message: impl Into<String>) -> SystemSettingsSmtpTestResponse {
    SystemSettingsSmtpTestResponse::failed(message)
}

pub fn sanitize_smtp_test_request(input: SystemSettingsSmtpTestRequest) -> SystemSettingsSmtpTestRequest {
    SystemSettingsSmtpTestRequest {
        smtp_host: trim_optional(input.smtp_host),
        smtp_username: trim_optional(input.smtp_username),
        smtp_password: trim_optional(input.smtp_password),
        smtp_from_email: trim_optional(input.smtp_from_email),
        smtp_from_name: trim_optional(input.smtp_from_name),
        ..input
    }
}

pub fn smtp_connection_config<C: SettingSecretCipher>(
    input: SystemSettingsSmtpTestRequest,
    stored: StoredSmtpSettings,
    cipher: &C,
) -> SettingResult<Result<SmtpConnectionConfig, String>> {
    let password = resolve_password(input.smtp_password, &stored.encrypted_smtp_password, cipher)?;
    let port = match clamp_port(input.smtp_port.unwrap_or(stored.smtp_port)) {
        Ok(port) => port,
        Err(message) => return Ok(Err(message)),
    };
    let config = SmtpConnectionConfig {
        host: input.smtp_host.unwrap_or(stored.smtp_host),
        port,
        username: input.smtp_username.unwrap_or(stored.smtp_username),
        password,
        from_email: input.smtp_from_email.unwrap_or(stored.smtp_from_email),
        from_name: input.smtp_from_name.unwrap_or(stored.smtp_from_name),
        encryption: input.smtp_encryption.unwrap_or(stored.smtp_encryption),
    };
    Ok(validate_config(config))
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned())
}

fn resolve_password<C: SettingSecretCipher>(input: Option<String>, stored: &str, cipher: &C) -> SettingResult<String> {
    match input {
        Some(password) => Ok(password),
        None if stored.is_empty() => Ok(String::new()),
        None => cipher.decrypt_secret(stored),
    }
}

fn clamp_port(value: i64) -> Result<u16, String> {
    if !(MIN_SMTP_PORT..=MAX_SMTP_PORT).contains(&value) {
        return Err(format!("SMTP 端口必须在 {MIN_SMTP_PORT} 到 {MAX_SMTP_PORT} 之间"));
    }
    Ok(value as u16)
}

fn validate_config(config: SmtpConnectionConfig) -> Result<SmtpConnectionConfig, String> {
    let missing = missing_fields(&config);
    if !missing.is_empty() {
        return Err(format!("{MISSING_SMTP_CONFIG_MESSAGE}{}", missing.join("、")));
    }
    validate_lengths(&config)?;
    validate_from_email(&config.from_email)?;
    Ok(config)
}

fn missing_fields(config: &SmtpConnectionConfig) -> Vec<&'static str> {
    [
        (config.host.is_empty(), "SMTP 服务器地址"),
        (config.username.is_empty(), "SMTP 用户名"),
        (config.password.is_empty(), "SMTP 密码"),
        (config.from_email.is_empty(), "发件人邮箱"),
    ]
    .into_iter()
    .filter_map(|(missing, label)| missing.then_some(label))
    .collect()
}

fn validate_lengths(config: &SmtpConnectionConfig) -> Result<(), String> {
    ensure_max("SMTP 服务器地址", &config.host, MAX_SMTP_HOST_LENGTH)?;
    ensure_max("SMTP 用户名", &config.username, MAX_SMTP_USERNAME_LENGTH)?;
    ensure_max("SMTP 密码", &config.password, MAX_SMTP_PASSWORD_LENGTH)?;
    ensure_max("发件人邮箱", &config.from_email, MAX_SMTP_FROM_EMAIL_LENGTH)?;
    ensure_max("发件人名称", &config.from_name, MAX_SMTP_FROM_NAME_LENGTH)
}

fn ensure_max(label: &str, value: &str, max: usize) -> Result<(), String> {
    if value.len() > max {
        return Err(format!("{label}长度不能超过 {max} 个字符"));
    }
    Ok(())
}

fn validate_from_email(value: &str) -> Result<(), String> {
    let Some((local, domain)) = value.split_once('@') else {
        return Err("发件人邮箱格式不正确".into());
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') || value.matches('@').count() != 1 {
        return Err("发件人邮箱格式不正确".into());
    }
    Ok(())
}
