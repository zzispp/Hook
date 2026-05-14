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
use types::{
    operations::SupportTicketEmailDelivery,
    system_setting::{SmtpEncryption, SystemSettings},
};

use crate::application::{TicketEmail, TicketMailer};

const EMAIL_FAILED: &str = "failed";
const EMAIL_DISABLED: &str = "disabled";
const EMAIL_SENT: &str = "sent";
const ERROR_EMAIL_CONFIGURATION_DISABLED: &str = "email_configuration_disabled";
const ERROR_SMTP_CONFIGURATION_INCOMPLETE: &str = "smtp_configuration_incomplete";
const SMTP_SEND_TIMEOUT_SECONDS: u64 = 30;

#[derive(Clone)]
pub struct SmtpTicketMailer<C> {
    settings: SettingStore,
    cipher: C,
}

impl<C> SmtpTicketMailer<C>
where
    C: SettingSecretCipher,
{
    pub fn new(database: Database, cipher: C) -> Self {
        Self {
            settings: SettingStore::new(database),
            cipher,
        }
    }

    async fn delivery_result(&self, email: TicketEmail) -> SupportTicketEmailDelivery {
        let settings = match self.settings.get_system_settings().await {
            Ok(settings) => settings,
            Err(error) => return delivery_failed(Some(error.to_string()), None),
        };
        if let Some(delivery) = email_readiness_delivery(&settings) {
            return delivery;
        }
        let smtp = match self.settings.get_smtp_settings().await {
            Ok(smtp) => smtp,
            Err(error) => return delivery_failed(Some(error.to_string()), None),
        };
        let password = match self.cipher.decrypt_secret(&smtp.encrypted_smtp_password) {
            Ok(password) => password,
            Err(error) => return delivery_failed(Some(error.to_string()), None),
        };
        send_email(EmailSendContext {
            settings: &settings,
            smtp: &smtp,
            password: &password,
            email,
        })
        .await
    }
}

#[async_trait]
impl<C> TicketMailer for SmtpTicketMailer<C>
where
    C: SettingSecretCipher,
{
    async fn send_ticket_email(&self, email: TicketEmail) -> SupportTicketEmailDelivery {
        self.delivery_result(email).await
    }
}

fn email_readiness_delivery(settings: &SystemSettings) -> Option<SupportTicketEmailDelivery> {
    if !settings.support_ticket_email_notifications_enabled {
        return Some(delivery_disabled());
    }
    if !settings.email_config_enabled {
        return Some(delivery_failed(None, Some(ERROR_EMAIL_CONFIGURATION_DISABLED)));
    }
    if settings.smtp_host.is_empty() || settings.smtp_username.is_empty() || !settings.smtp_password_set || settings.smtp_from_email.is_empty() {
        return Some(delivery_failed(None, Some(ERROR_SMTP_CONFIGURATION_INCOMPLETE)));
    }
    None
}

struct EmailSendContext<'a> {
    settings: &'a SystemSettings,
    smtp: &'a storage::setting::SystemSettingsSmtpRecord,
    password: &'a str,
    email: TicketEmail,
}

async fn send_email(context: EmailSendContext<'_>) -> SupportTicketEmailDelivery {
    let transport = match smtp_transport(context.smtp, context.password) {
        Ok(transport) => transport,
        Err(error) => return delivery_failed(Some(error), None),
    };
    let message = match email_message(&context.settings.smtp_from_email, &context.settings.smtp_from_name, context.email) {
        Ok(message) => message,
        Err(error) => return delivery_failed(Some(error), None),
    };
    match transport.send(message).await {
        Ok(_) => delivery_sent(),
        Err(error) => delivery_failed(Some(error.to_string()), None),
    }
}

fn smtp_transport(smtp: &storage::setting::SystemSettingsSmtpRecord, password: &str) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp.smtp_host)
        .port(smtp.smtp_port as u16)
        .timeout(Some(Duration::from_secs(SMTP_SEND_TIMEOUT_SECONDS)))
        .tls(tls_mode(&smtp.smtp_host, smtp.smtp_encryption)?)
        .credentials(Credentials::new(smtp.smtp_username.clone(), password.to_owned()));
    Ok(builder.build())
}

fn tls_mode(host: &str, encryption: SmtpEncryption) -> Result<Tls, String> {
    match encryption {
        SmtpEncryption::None => Ok(Tls::None),
        SmtpEncryption::Tls => tls_parameters(host).map(Tls::Required),
        SmtpEncryption::Ssl => tls_parameters(host).map(Tls::Wrapper),
    }
}

fn tls_parameters(host: &str) -> Result<TlsParameters, String> {
    TlsParameters::new(host.to_owned()).map_err(|error| format!("TLS parameters are invalid: {error}"))
}

fn email_message(from_email: &str, from_name: &str, email: TicketEmail) -> Result<Message, String> {
    Message::builder()
        .from(mailbox(from_name, from_email)?)
        .to(mailbox("", &email.recipient_email)?)
        .subject(email.subject)
        .header(ContentType::TEXT_HTML)
        .body(markdown_email_body(&email.body_markdown))
        .map_err(|error| error.to_string())
}

fn mailbox(name: &str, email: &str) -> Result<Mailbox, String> {
    let address = email.parse().map_err(|error| format!("email address is invalid: {error}"))?;
    let name = if name.trim().is_empty() { None } else { Some(name.trim().to_owned()) };
    Ok(Mailbox::new(name, address))
}

fn markdown_email_body(markdown: &str) -> String {
    let escaped = escape_html(markdown);
    format!("<pre style=\"white-space:pre-wrap;font-family:inherit\">{escaped}</pre>")
}

fn escape_html(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn delivery_sent() -> SupportTicketEmailDelivery {
    delivery(EMAIL_SENT, None, None)
}

fn delivery_disabled() -> SupportTicketEmailDelivery {
    delivery(EMAIL_DISABLED, None, None)
}

fn delivery_failed(error_message: Option<String>, error_code: Option<&str>) -> SupportTicketEmailDelivery {
    delivery(EMAIL_FAILED, error_message, error_code)
}

fn delivery(status: &str, error_message: Option<String>, error_code: Option<&str>) -> SupportTicketEmailDelivery {
    SupportTicketEmailDelivery {
        status: status.into(),
        error_code: error_code.map(str::to_owned),
        error_message,
    }
}
