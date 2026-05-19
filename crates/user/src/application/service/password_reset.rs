use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use crate::application::{
    AppError, AppResult, EmailSettings, PasswordResetConfig, PasswordResetEmail, PasswordResetRecord, PasswordResetRepository, PasswordResetRequest,
    PasswordResetTemplate,
};
use types::user::User;

const RESET_TOKEN_BYTES: usize = 32;
const PASSWORD_RESET_EXPIRE_MINUTES: i64 = 30;
const PASSWORD_RESET_PATH: &str = "/auth/reset-password";
const PASSWORD_RESET_DISABLED: &str = "password reset is disabled";
const EMAIL_CONFIGURATION_DISABLED: &str = "email configuration is disabled";
const SMTP_CONFIGURATION_INCOMPLETE: &str = "SMTP configuration is incomplete";

pub(super) async fn request_password_reset<R, C, M>(repository: &R, config: &C, mailer: &M, input: PasswordResetRequest) -> AppResult<()>
where
    R: PasswordResetRepository + crate::application::UserRepository,
    C: PasswordResetConfig,
    M: crate::application::PasswordResetMailer,
{
    let settings = config.password_reset_settings().await?;
    reject_unready_email_config(&settings)?;
    let template = config.password_reset_template(&input.lang).await?;
    let Some(user) = repository.find_by_email(&input.email).await? else {
        return Ok(());
    };
    let token = random_token();
    let expires_at = time::OffsetDateTime::now_utc() + time::Duration::minutes(PASSWORD_RESET_EXPIRE_MINUTES);
    repository
        .create_password_reset_token(PasswordResetRecord {
            user_id: user.id.clone(),
            token_hash: token_hash(&token),
            expires_at,
        })
        .await?;
    mailer.send_password_reset(email(settings, template, input, token)).await
}

pub(super) async fn reset_password<R>(repository: &R, token: &str, password_hash: &str) -> AppResult<User>
where
    R: PasswordResetRepository,
{
    repository
        .consume_password_reset_token(&token_hash(token), password_hash, time::OffsetDateTime::now_utc())
        .await?
        .ok_or_else(|| AppError::InvalidInput("password reset token is invalid or expired".into()))
}

fn email(settings: EmailSettings, template: PasswordResetTemplate, input: PasswordResetRequest, token: String) -> PasswordResetEmail {
    let recipient_email = input.email.clone();
    let variables = PasswordResetVariables {
        app_name: &settings.site_name,
        email: &input.email,
        expire_minutes: PASSWORD_RESET_EXPIRE_MINUTES,
        reset_link: reset_link(&input.reset_origin, &token),
    };
    PasswordResetEmail {
        recipient_email,
        subject: render_template(&template.subject, &variables),
        html: render_template(&template.html, &variables),
        settings,
    }
}

fn reject_unready_email_config(settings: &EmailSettings) -> AppResult<()> {
    if !settings.feature_enabled {
        return Err(AppError::InvalidInput(PASSWORD_RESET_DISABLED.into()));
    }
    if !settings.email_config_enabled {
        return Err(AppError::InvalidInput(EMAIL_CONFIGURATION_DISABLED.into()));
    }
    if settings.smtp_host.is_empty() || settings.smtp_username.is_empty() || !settings.smtp_password_set || settings.smtp_from_email.is_empty() {
        return Err(AppError::InvalidInput(SMTP_CONFIGURATION_INCOMPLETE.into()));
    }
    Ok(())
}

struct PasswordResetVariables<'a> {
    app_name: &'a str,
    email: &'a str,
    expire_minutes: i64,
    reset_link: String,
}

fn render_template(template: &str, variables: &PasswordResetVariables<'_>) -> String {
    template
        .replace("{{app_name}}", variables.app_name)
        .replace("{{email}}", variables.email)
        .replace("{{expire_minutes}}", &variables.expire_minutes.to_string())
        .replace("{{reset_link}}", &variables.reset_link)
}

fn reset_link(origin: &str, token: &str) -> String {
    format!("{origin}{PASSWORD_RESET_PATH}?token={token}")
}

pub(super) fn token_hash(token: &str) -> String {
    hex(Sha256::digest(token.as_bytes()).as_ref())
}

fn random_token() -> String {
    let mut bytes = [0_u8; RESET_TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    hex(&bytes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_hash_is_sha256_hex() {
        let hash = token_hash("token-value");

        assert_eq!(hash.len(), 64);
        assert_eq!(hash, "e6c02a5742ea9d4de588eb9b9de7bed43dc17011552186bed3e98b2c5958ff4a");
    }
}
