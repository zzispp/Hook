use rust_decimal::Decimal;
use types::user::{User, UserIdentitySummary};

use crate::application::{AppError, AppResult, InitialGrantLedger, PasswordHasher, UserAuthRecord, UserRepository};

pub(super) async fn grant_initial_balance<G>(ledger: &G, user: &User, amount: Decimal) -> AppResult<()>
where
    G: InitialGrantLedger,
{
    if amount <= Decimal::ZERO {
        return Ok(());
    }
    ledger.grant_initial_balance(&user.id.0, amount).await
}

pub(super) fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    let Some(password_hash) = found.password_hash.as_deref() else {
        return Err(AppError::PasswordNotSet);
    };
    if hasher.verify(password, password_hash)? {
        return Ok(());
    }
    Err(AppError::InvalidCredentials)
}

pub(super) async fn unlink_identity<R>(repository: &R, user: &User, identity_id: &str) -> AppResult<()>
where
    R: UserRepository,
{
    let identities = repository.list_identities_by_user_id(&user.id.0).await?;
    if !identities.iter().any(|identity| identity.id == identity_id) {
        return Err(AppError::NotFound);
    }
    if !user.password_set && identities.len() <= 1 {
        return Err(AppError::InvalidInput("at least one login method must remain".into()));
    }
    repository.delete_identity(identity_id).await
}

pub(super) fn identity_summaries(identities: Vec<types::user::UserIdentity>) -> Vec<UserIdentitySummary> {
    identities.into_iter().map(UserIdentitySummary::from).collect()
}
