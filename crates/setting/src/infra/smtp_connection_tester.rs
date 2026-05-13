use std::time::Duration;

use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, Tokio1Executor,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
};
use types::system_setting::SmtpEncryption;

use crate::application::{SmtpConnectionConfig, SmtpConnectionTester};

const SMTP_TEST_TIMEOUT_SECONDS: u64 = 30;

#[derive(Clone, Copy)]
pub struct LettreSmtpConnectionTester;

#[async_trait]
impl SmtpConnectionTester for LettreSmtpConnectionTester {
    async fn test_connection(&self, config: &SmtpConnectionConfig) -> Result<(), String> {
        let transport = smtp_transport(config)?;
        match transport.test_connection().await {
            Ok(true) => Ok(()),
            Ok(false) => Err("SMTP 服务器未确认连接可用".into()),
            Err(error) => Err(translate_smtp_error(&error.to_string())),
        }
    }
}

fn smtp_transport(config: &SmtpConnectionConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
        .port(config.port)
        .timeout(Some(Duration::from_secs(SMTP_TEST_TIMEOUT_SECONDS)))
        .tls(tls_mode(config)?)
        .credentials(Credentials::new(config.username.clone(), config.password.clone()));
    Ok(builder.build())
}

fn tls_mode(config: &SmtpConnectionConfig) -> Result<Tls, String> {
    match config.encryption {
        SmtpEncryption::None => Ok(Tls::None),
        SmtpEncryption::Tls => tls_parameters(config).map(Tls::Required),
        SmtpEncryption::Ssl => tls_parameters(config).map(Tls::Wrapper),
    }
}

fn tls_parameters(config: &SmtpConnectionConfig) -> Result<TlsParameters, String> {
    TlsParameters::new(config.host.clone()).map_err(|error| format!("TLS 参数无效：{error}"))
}

fn translate_smtp_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("authentication") || lower.contains("credentials") || lower.contains("password") {
        return "SMTP 认证失败，请检查用户名和密码".into();
    }
    if lower.contains("connection refused") {
        return "SMTP 连接被拒绝，请检查服务器地址和端口".into();
    }
    if lower.contains("timed out") || lower.contains("timeout") {
        return "SMTP 连接超时，请检查网络、服务器地址或端口".into();
    }
    if lower.contains("dns") || lower.contains("resolve") || lower.contains("name") {
        return "无法解析 SMTP 服务器地址".into();
    }
    if lower.contains("starttls") || lower.contains("tls") || lower.contains("certificate") {
        return "SMTP 加密连接失败，请检查加密方式和端口".into();
    }
    format!("SMTP 连接测试失败：{error}")
}
