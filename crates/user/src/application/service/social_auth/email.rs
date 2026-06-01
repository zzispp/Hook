use rand_core::{OsRng, RngCore};
use types::user::User;

use crate::application::{
    AppError, AppResult, PurposeEmailCodeStore, RegistrationEmailCodeRequest, RegistrationEmailConfig, RegistrationEmailMailer, UserAuthRecord,
    UserRepository,
};

use super::helpers::{email_verified_record, password_replace_record};
use crate::application::PasswordHasher;
use crate::application::RegistrationEmail;

use super::super::validation::{
    sanitize_registration_email_code_request, validate_email_verification_code, validate_password, validate_registration_email_code_request,
};

pub(in crate::application::service) const ACCOUNT_PASSWORD_EMAIL_PURPOSE: &str = "account_password";

const EMAIL_CODE_TTL_SECONDS: u64 = 10 * 60;
const EMAIL_CODE_COOLDOWN_SECONDS: u64 = 60;
const EMAIL_CODE_BYTES: usize = 4;
const EMAIL_CODE_MODULO: u32 = 1_000_000;
const EMAIL_FEATURE_DISABLED: &str = "email verification is disabled";
const EMAIL_CONFIGURATION_DISABLED: &str = "email configuration is disabled";
const SMTP_CONFIGURATION_INCOMPLETE: &str = "SMTP configuration is incomplete";

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
    let user_id = user.id.clone();
    let mut record = password_replace_record(user, password_hash);
    record.email_verified = Some(true);
    repository.replace(user_id, record).await
}

pub(in crate::application::service) async fn change_password_with_current_password<R, H>(
    repository: &R,
    hasher: &H,
    user_auth: UserAuthRecord,
    current_password: &str,
    password: &str,
) -> AppResult<User>
where
    R: UserRepository,
    H: PasswordHasher,
{
    validate_password(current_password)?;
    validate_password(password)?;
    let Some(password_hash) = user_auth.password_hash.as_deref() else {
        return Err(AppError::PasswordNotSet);
    };
    if !hasher.verify(current_password, password_hash)? {
        return Err(AppError::InvalidCredentials);
    }
    let password_hash = hasher.hash(password)?;
    let user_id = user_auth.user.id.clone();
    repository.replace(user_id, password_replace_record(user_auth.user, password_hash)).await
}

pub(in crate::application::service) async fn verify_email_with_code<R, S>(repository: &R, codes: &S, user: User, code: &str) -> AppResult<User>
where
    R: UserRepository,
    S: PurposeEmailCodeStore,
{
    validate_email_verification_code(code)?;
    if !codes.consume_email_code(ACCOUNT_PASSWORD_EMAIL_PURPOSE, &user.email, code).await? {
        return Err(AppError::InvalidInput("email verification code is invalid or expired".into()));
    }
    repository.replace(user.id.clone(), email_verified_record(user)).await
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
    let settings = config.account_email_settings().await?;
    reject_unready_email_config(&settings)?;
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

fn reject_unready_email_config(settings: &crate::application::EmailSettings) -> AppResult<()> {
    if !settings.feature_enabled {
        return Err(AppError::InvalidInput(EMAIL_FEATURE_DISABLED.into()));
    }
    if !settings.email_config_enabled {
        return Err(AppError::InvalidInput(EMAIL_CONFIGURATION_DISABLED.into()));
    }
    if !settings.is_ready() {
        return Err(AppError::InvalidInput(SMTP_CONFIGURATION_INCOMPLETE.into()));
    }
    Ok(())
}

fn random_code() -> String {
    let mut bytes = [0_u8; EMAIL_CODE_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let value = u32::from_be_bytes(bytes) % EMAIL_CODE_MODULO;
    format!("{value:06}")
}
