use std::time::Duration;

use lettre::{
    AsyncSmtpTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
};
use types::system_setting::SmtpEncryption;

use crate::application::{AppError, AppResult};

const SMTP_SEND_TIMEOUT_SECONDS: u64 = 30;

pub(super) fn smtp_transport(
    host: &str,
    port: i64,
    encryption: SmtpEncryption,
    username: &str,
    password: &str,
) -> AppResult<AsyncSmtpTransport<Tokio1Executor>> {
    let port = u16::try_from(port).map_err(|_| AppError::InvalidInput("SMTP port is invalid".into()))?;
    let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
        .port(port)
        .timeout(Some(Duration::from_secs(SMTP_SEND_TIMEOUT_SECONDS)))
        .tls(tls_mode(host, encryption)?)
        .credentials(Credentials::new(username.to_owned(), password.to_owned()));
    Ok(builder.build())
}

pub(super) fn email_message(from_email: &str, from_name: &str, recipient: &str, subject: &str, html: String) -> AppResult<Message> {
    Message::builder()
        .from(mailbox(from_name, from_email)?)
        .to(mailbox("", recipient)?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
        .map_err(|error| AppError::Infrastructure(error.to_string()))
}

pub(super) fn smtp_error(error: lettre::transport::smtp::Error) -> AppError {
    AppError::Infrastructure(format!("SMTP send failed: {error}"))
}

fn tls_mode(host: &str, encryption: SmtpEncryption) -> AppResult<Tls> {
    match encryption {
        SmtpEncryption::None => Ok(Tls::None),
        SmtpEncryption::Tls => tls_parameters(host).map(Tls::Required),
        SmtpEncryption::Ssl => tls_parameters(host).map(Tls::Wrapper),
    }
}

fn tls_parameters(host: &str) -> AppResult<TlsParameters> {
    TlsParameters::new(host.to_owned()).map_err(|error| AppError::Infrastructure(format!("TLS parameters are invalid: {error}")))
}

fn mailbox(name: &str, email: &str) -> AppResult<Mailbox> {
    let address = email
        .parse()
        .map_err(|error| AppError::InvalidInput(format!("email address is invalid: {error}")))?;
    let name = if name.trim().is_empty() { None } else { Some(name.trim().to_owned()) };
    Ok(Mailbox::new(name, address))
}
