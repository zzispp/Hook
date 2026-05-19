use rand_core::{OsRng, RngCore};
use types::user::{RegistrationEmailCodeRequest, SignUpUser};

use crate::application::{
    AppError, AppResult, EmailSettings, RegistrationEmail, RegistrationEmailCodeStore, RegistrationEmailConfig, RegistrationEmailMailer,
    RegistrationEmailTemplate, RegistrationSettings,
};

use crate::application::service::validation::validate_email_verification_code;

const REGISTRATION_CODE_BYTES: usize = 4;
const REGISTRATION_CODE_MODULO: u32 = 1_000_000;
const REGISTRATION_CODE_TTL_SECONDS: u64 = 10 * 60;
const REGISTRATION_CODE_EXPIRE_MINUTES: u64 = REGISTRATION_CODE_TTL_SECONDS / 60;
const REGISTRATION_CODE_RESEND_COOLDOWN_SECONDS: u64 = 60;
const REGISTRATION_EMAIL_DISABLED: &str = "registration email verification is disabled";
const EMAIL_CONFIGURATION_DISABLED: &str = "email configuration is disabled";
const SMTP_CONFIGURATION_INCOMPLETE: &str = "SMTP configuration is incomplete";
const REGISTRATION_CODE_RATE_LIMITED: &str = "registration email code can only be requested once every 60 seconds";

pub(in crate::application::service) async fn request_registration_email_code<S, C, M>(
    code_store: &S,
    config: &C,
    mailer: &M,
    input: RegistrationEmailCodeRequest,
) -> AppResult<()>
where
    S: RegistrationEmailCodeStore,
    C: RegistrationEmailConfig,
    M: RegistrationEmailMailer,
{
    let settings = config.registration_email_settings().await?;
    reject_unready_email_config(&settings)?;
    let template = config.registration_email_template(&input.lang).await?;
    let code = registration_code(code_store, &input.email).await?;
    mailer.send_registration_email(email(settings, template, input, code)).await
}

pub(in crate::application::service) async fn verify_registration_email_code<S>(
    code_store: &S,
    settings: &RegistrationSettings,
    input: &SignUpUser,
) -> AppResult<()>
where
    S: RegistrationEmailCodeStore,
{
    if !settings.registration_email_verification_enabled {
        return Ok(());
    }
    let code = required_code(input)?;
    validate_email_verification_code(code)?;
    let consumed = code_store.consume_registration_email_code(&input.user.email.to_ascii_lowercase(), code).await?;
    if consumed {
        return Ok(());
    }
    Err(AppError::InvalidInput("email verification code is invalid or expired".into()))
}

async fn registration_code<S>(code_store: &S, email: &str) -> AppResult<String>
where
    S: RegistrationEmailCodeStore,
{
    if !code_store
        .begin_registration_email_code_cooldown(email, REGISTRATION_CODE_RESEND_COOLDOWN_SECONDS)
        .await?
    {
        return Err(AppError::InvalidInput(REGISTRATION_CODE_RATE_LIMITED.into()));
    }
    if let Some(code) = code_store.active_registration_email_code(email).await? {
        return Ok(code);
    }
    let code = random_code();
    code_store.save_registration_email_code(email, &code, REGISTRATION_CODE_TTL_SECONDS).await?;
    Ok(code)
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
    expire_minutes: u64,
}

fn render_template(template: &str, variables: &RegistrationEmailVariables<'_>) -> String {
    template
        .replace("{{app_name}}", variables.app_name)
        .replace("{{email}}", variables.email)
        .replace("{{code}}", variables.code)
        .replace("{{expire_minutes}}", &variables.expire_minutes.to_string())
}

fn random_code() -> String {
    let mut bytes = [0_u8; REGISTRATION_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let value = u32::from_be_bytes(bytes) % REGISTRATION_CODE_MODULO;
    format!("{value:06}")
}
