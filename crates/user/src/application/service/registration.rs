use types::system_setting::EmailSuffixMode;

use crate::application::{AppError, AppResult, RegistrationSettings};

const EMAIL_SUFFIX_SEPARATOR: char = ',';

pub(super) fn reject_closed_registration(settings: &RegistrationSettings) -> AppResult<()> {
    if settings.allow_registration {
        return Ok(());
    }
    Err(AppError::InvalidInput("registration is closed".into()))
}

pub(super) fn reject_disallowed_registration_email(settings: &RegistrationSettings, email: &str) -> AppResult<()> {
    match settings.email_suffix_mode {
        EmailSuffixMode::None => Ok(()),
        EmailSuffixMode::Whitelist => reject_by_whitelist(email, &settings.email_suffixes),
        EmailSuffixMode::Blacklist => reject_by_blacklist(email, &settings.email_suffixes),
    }
}

fn reject_by_whitelist(email: &str, suffixes: &str) -> AppResult<()> {
    let suffixes = parsed_suffixes(suffixes)?;
    if suffix_matches(email, &suffixes)? {
        return Ok(());
    }
    Err(disallowed_suffix())
}

fn reject_by_blacklist(email: &str, suffixes: &str) -> AppResult<()> {
    let suffixes = parsed_suffixes(suffixes)?;
    if suffix_matches(email, &suffixes)? {
        return Err(disallowed_suffix());
    }
    Ok(())
}

fn parsed_suffixes(value: &str) -> AppResult<Vec<&str>> {
    let suffixes = value
        .split(EMAIL_SUFFIX_SEPARATOR)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    if suffixes.is_empty() {
        return Err(AppError::InvalidInput("email suffix restriction list cannot be empty".into()));
    }
    Ok(suffixes)
}

fn suffix_matches(email: &str, suffixes: &[&str]) -> AppResult<bool> {
    let domain = email_domain(email)?;
    Ok(suffixes.iter().any(|suffix| domain == suffix.to_ascii_lowercase()))
}

fn email_domain(email: &str) -> AppResult<String> {
    let Some((local, domain)) = email.split_once('@') else {
        return Err(AppError::InvalidInput("email must contain a domain for suffix restriction".into()));
    };
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return Err(AppError::InvalidInput("email must contain a domain for suffix restriction".into()));
    }
    Ok(domain.to_ascii_lowercase())
}

fn disallowed_suffix() -> AppError {
    AppError::InvalidInput("email suffix is not allowed for registration".into())
}
