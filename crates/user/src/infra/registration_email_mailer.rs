use async_trait::async_trait;
use lettre::AsyncTransport;
use setting::application::SettingSecretCipher;
use storage::{Database, setting::SettingStore};

use crate::application::{AppError, AppResult, RegistrationEmail, RegistrationEmailMailer};

use super::smtp_mailer::{email_message, smtp_error, smtp_transport};

#[derive(Clone)]
pub struct SmtpRegistrationEmailMailer<C> {
    settings: SettingStore,
    cipher: C,
}

impl<C> SmtpRegistrationEmailMailer<C>
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
impl<C> RegistrationEmailMailer for SmtpRegistrationEmailMailer<C>
where
    C: SettingSecretCipher,
{
    async fn send_registration_email(&self, email: RegistrationEmail) -> AppResult<()> {
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

fn storage_error(error: storage::StorageError) -> AppError {
    match error {
        storage::StorageError::NotFound => AppError::NotFound,
        storage::StorageError::Conflict(message) | storage::StorageError::Database(message) => AppError::Infrastructure(message),
    }
}

fn setting_error(error: setting::application::SettingError) -> AppError {
    AppError::Infrastructure(error.to_string())
}
