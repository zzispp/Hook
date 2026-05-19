use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use types::user::{RegistrationEmailCodeRequest, SignUpUser};

use crate::application::{
    AppError, AppResult, EmailSettings, RegistrationEmail, RegistrationEmailConfig, RegistrationEmailMailer, RegistrationEmailRepository,
    RegistrationEmailTemplate, RegistrationEmailVerificationRecord, RegistrationSettings,
};

use crate::application::service::validation::validate_email_verification_code;

const REGISTRATION_CODE_BYTES: usize = 4;
const REGISTRATION_CODE_MODULO: u32 = 1_000_000;
const REGISTRATION_CODE_EXPIRE_MINUTES: i64 = 10;
const REGISTRATION_EMAIL_DISABLED: &str = "registration email verification is disabled";
const EMAIL_CONFIGURATION_DISABLED: &str = "email configuration is disabled";
const SMTP_CONFIGURATION_INCOMPLETE: &str = "SMTP configuration is incomplete";

pub(in crate::application::service) async fn request_registration_email_code<R, C, M>(
    repository: &R,
    config: &C,
    mailer: &M,
    input: RegistrationEmailCodeRequest,
) -> AppResult<()>
where
    R: RegistrationEmailRepository,
    C: RegistrationEmailConfig,
    M: RegistrationEmailMailer,
{
    let settings = config.registration_email_settings().await?;
    reject_unready_email_config(&settings)?;
    let template = config.registration_email_template(&input.lang).await?;
    let code = random_code();
    repository
        .create_registration_email_verification(verification_record(&input.email, &code))
        .await?;
    mailer.send_registration_email(email(settings, template, input, code)).await
}

pub(in crate::application::service) async fn verify_registration_email_code<R>(
    repository: &R,
    settings: &RegistrationSettings,
    input: &SignUpUser,
) -> AppResult<()>
where
    R: RegistrationEmailRepository,
{
    if !settings.registration_email_verification_enabled {
        return Ok(());
    }
    let code = required_code(input)?;
    validate_email_verification_code(code)?;
    let consumed = repository
        .consume_registration_email_verification(&input.user.email, &code_hash(code), time::OffsetDateTime::now_utc())
        .await?;
    if consumed {
        return Ok(());
    }
    Err(AppError::InvalidInput("email verification code is invalid or expired".into()))
}

fn verification_record(email: &str, code: &str) -> RegistrationEmailVerificationRecord {
    RegistrationEmailVerificationRecord {
        email: email.into(),
        code_hash: code_hash(code),
        expires_at: time::OffsetDateTime::now_utc() + time::Duration::minutes(REGISTRATION_CODE_EXPIRE_MINUTES),
    }
}

fn required_code(input: &SignUpUser) -> AppResult<&str> {
    input
        .email_verification_code
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::InvalidInput("email verification code is required".into()))
}

fn email(settings: EmailSettings, template: RegistrationEmailTemplate, input: RegistrationEmailCodeRequest, code: String) -> RegistrationEmail {
    let recipient_email = input.email.clone();
    let variables = RegistrationEmailVariables {
        app_name: &settings.site_name,
        email: &input.email,
        code: &code,
        expire_minutes: REGISTRATION_CODE_EXPIRE_MINUTES,
    };
    RegistrationEmail {
        recipient_email,
        subject: render_template(&template.subject, &variables),
        html: render_template(&template.html, &variables),
        settings,
    }
}

fn reject_unready_email_config(settings: &EmailSettings) -> AppResult<()> {
    if !settings.feature_enabled {
        return Err(AppError::InvalidInput(REGISTRATION_EMAIL_DISABLED.into()));
    }
    if !settings.email_config_enabled {
        return Err(AppError::InvalidInput(EMAIL_CONFIGURATION_DISABLED.into()));
    }
    if settings.smtp_host.is_empty() || settings.smtp_username.is_empty() || !settings.smtp_password_set || settings.smtp_from_email.is_empty() {
        return Err(AppError::InvalidInput(SMTP_CONFIGURATION_INCOMPLETE.into()));
    }
    Ok(())
}

struct RegistrationEmailVariables<'a> {
    app_name: &'a str,
    email: &'a str,
    code: &'a str,
    expire_minutes: i64,
}

fn render_template(template: &str, variables: &RegistrationEmailVariables<'_>) -> String {
    template
        .replace("{{app_name}}", variables.app_name)
        .replace("{{email}}", variables.email)
        .replace("{{code}}", variables.code)
        .replace("{{expire_minutes}}", &variables.expire_minutes.to_string())
}

fn code_hash(code: &str) -> String {
    hex(Sha256::digest(code.as_bytes()).as_ref())
}

fn random_code() -> String {
    let mut bytes = [0_u8; REGISTRATION_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let value = u32::from_be_bytes(bytes) % REGISTRATION_CODE_MODULO;
    format!("{value:06}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
