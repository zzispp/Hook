use rand_core::{OsRng, RngCore};
use types::user::User;

use crate::application::{
    AppError, AppResult, PurposeEmailCodeStore, RegistrationEmailCodeRequest, RegistrationEmailConfig, RegistrationEmailMailer, UserRepository,
};

use super::helpers::password_replace_record;
use crate::application::PasswordHasher;
use crate::application::RegistrationEmail;

use super::super::validation::{
    sanitize_registration_email_code_request, validate_email_verification_code, validate_password, validate_registration_email_code_request,
};

pub(in crate::application::service) const ACCOUNT_PASSWORD_EMAIL_PURPOSE: &str = "account_password";
pub(in crate::application::service) const WALLET_EMAIL_PURPOSE: &str = "wallet_binding";

const EMAIL_CODE_TTL_SECONDS: u64 = 10 * 60;
const EMAIL_CODE_COOLDOWN_SECONDS: u64 = 60;
const EMAIL_CODE_BYTES: usize = 4;
const EMAIL_CODE_MODULO: u32 = 1_000_000;

pub(in crate::application::service) async fn request_purpose_email_code<S, C, M>(
    store: &S,
    config: &C,
    mailer: &M,
    purpose: &str,
    input: RegistrationEmailCodeRequest,
) -> AppResult<()>
where
    S: PurposeEmailCodeStore,
    C: RegistrationEmailConfig,
    M: RegistrationEmailMailer,
{
    let input = sanitize_registration_email_code_request(input);
    validate_registration_email_code_request(&input)?;
    let code = purpose_email_code(store, purpose, &input.email).await?;
    request_registration_email_code_with_code(config, mailer, input, code).await
}

pub(in crate::application::service) async fn change_password_with_email_code<R, S, H>(
    repository: &R,
    codes: &S,
    hasher: &H,
    user: User,
    code: &str,
    password: &str,
) -> AppResult<User>
where
    R: UserRepository,
    S: PurposeEmailCodeStore,
    H: PasswordHasher,
{
    validate_email_verification_code(code)?;
    validate_password(password)?;
    if !codes.consume_email_code(ACCOUNT_PASSWORD_EMAIL_PURPOSE, &user.email, code).await? {
        return Err(AppError::InvalidInput("email verification code is invalid or expired".into()));
    }
    let password_hash = hasher.hash(password)?;
    repository.replace(user.id.clone(), password_replace_record(user, password_hash)).await
}

async fn purpose_email_code<S>(store: &S, purpose: &str, email: &str) -> AppResult<String>
where
    S: PurposeEmailCodeStore,
{
    if !store.begin_email_code_cooldown(purpose, email, EMAIL_CODE_COOLDOWN_SECONDS).await? {
        return Err(AppError::InvalidInput("email code can only be requested once every 60 seconds".into()));
    }
    if let Some(code) = store.active_email_code(purpose, email).await? {
        return Ok(code);
    }
    let code = random_code();
    store.save_email_code(purpose, email, &code, EMAIL_CODE_TTL_SECONDS).await?;
    Ok(code)
}

async fn request_registration_email_code_with_code<C, M>(config: &C, mailer: &M, input: RegistrationEmailCodeRequest, code: String) -> AppResult<()>
where
    C: RegistrationEmailConfig,
    M: RegistrationEmailMailer,
{
    let settings = config.registration_email_settings().await?;
    if settings.smtp_host.is_empty() || settings.smtp_username.is_empty() || !settings.smtp_password_set || settings.smtp_from_email.is_empty() {
        return Err(AppError::InvalidInput("SMTP configuration is incomplete".into()));
    }
    let template = config.registration_email_template(&input.lang).await?;
    let html = template
        .html
        .replace("{{app_name}}", &settings.site_name)
        .replace("{{email}}", &input.email)
        .replace("{{code}}", &code)
        .replace("{{expire_minutes}}", "10");
    let subject = template.subject.replace("{{code}}", &code);
    mailer
        .send_registration_email(RegistrationEmail {
            recipient_email: input.email,
            subject,
            html,
            settings,
        })
        .await
}

fn random_code() -> String {
    let mut bytes = [0_u8; EMAIL_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let value = u32::from_be_bytes(bytes) % EMAIL_CODE_MODULO;
    format!("{value:06}")
}
