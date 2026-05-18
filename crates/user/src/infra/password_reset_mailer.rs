use std::time::Duration;

use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
};
use setting::application::SettingSecretCipher;
use storage::{Database, setting::SettingStore};
use types::system_setting::SmtpEncryption;

use crate::application::{AppError, AppResult, PasswordResetEmail, PasswordResetMailer};

const SMTP_SEND_TIMEOUT_SECONDS: u64 = 30;

#[derive(Clone)]
pub struct SmtpPasswordResetMailer<C> {
    settings: SettingStore,
    cipher: C,
}

impl<C> SmtpPasswordResetMailer<C>
where
    C: SettingSecretCipher,
{
    pub fn new(database: Database, cipher: C) -> Self {
        Self {
            settings: SettingStore::new(database),
            cipher,
        }
    }
}

#[async_trait]
impl<C> PasswordResetMailer for SmtpPasswordResetMailer<C>
where
    C: SettingSecretCipher,
{
    async fn send_password_reset(&self, email: PasswordResetEmail) -> AppResult<()> {
        let smtp = self.settings.get_smtp_settings().await.map_err(storage_error)?;
        let password = self.cipher.decrypt_secret(&smtp.encrypted_smtp_password).map_err(setting_error)?;
        let transport = smtp_transport(&smtp.smtp_host, smtp.smtp_port, smtp.smtp_encryption, &smtp.smtp_username, &password)?;
        let message = email_message(
            &email.settings.smtp_from_email,
            &email.settings.smtp_from_name,
            &email.recipient_email,
            &email.subject,
            email.html,
        )?;
        transport.send(message).await.map(|_| ()).map_err(smtp_error)
    }
}

fn smtp_transport(
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

fn email_message(from_email: &str, from_name: &str, recipient: &str, subject: &str, html: String) -> AppResult<Message> {
    Message::builder()
        .from(mailbox(from_name, from_email)?)
        .to(mailbox("", recipient)?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html)
        .map_err(|error| AppError::Infrastructure(error.to_string()))
}

fn mailbox(name: &str, email: &str) -> AppResult<Mailbox> {
    let address = email
        .parse()
        .map_err(|error| AppError::InvalidInput(format!("email address is invalid: {error}")))?;
    let name = if name.trim().is_empty() { None } else { Some(name.trim().to_owned()) };
    Ok(Mailbox::new(name, address))
}

fn storage_error(error: storage::StorageError) -> AppError {
    match error {
        storage::StorageError::NotFound => AppError::NotFound,
        storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

fn setting_error(error: setting::application::SettingError) -> AppError {
    AppError::Infrastructure(error.to_string())
}

fn smtp_error(error: lettre::transport::smtp::Error) -> AppError {
    AppError::Infrastructure(format!("SMTP send failed: {error}"))
}
