use provider::application::billing::RequestBillingAmount;
use rust_decimal::Decimal;
use storage::{
    StorageError,
    api_token::ApiTokenStore,
    wallet::{WALLET_CONSUME_INSUFFICIENT_BALANCE, WalletConsumeRecordInput, WalletStore},
};
use types::{
    api_token::{ApiToken, ApiTokenType},
    user::{USER_QUOTA_MODE_UNLIMITED, USER_QUOTA_MODE_WALLET},
    wallet::Wallet,
};

mod snapshot;

use super::{LlmProxyError, LlmProxyState, audit::TokenUsage, cache::snapshot::CachedUserAccess, candidate::ProxyCandidate};

const CATEGORY_CONSUME: &str = "consume";
const ERROR_CODE_DISABLED_USER: &str = "new_api_error";
const ERROR_CODE_TOKEN_QUOTA: &str = "pre_consume_token_quota_failed";
const ERROR_CODE_WALLET_QUOTA: &str = "insufficient_user_quota";
const LINK_LLM_REQUEST_RECORD: &str = "llm_request_record";
const REASON_LLM_MODEL_USAGE: &str = "llm_model_usage";
const WALLET_STATUS_ACTIVE: &str = "active";
const WALLET_LIMIT_UNLIMITED: &str = "unlimited";

pub(super) struct WalletSettlementInput<'a> {
    pub(super) request_id: &'a str,
    pub(super) candidate: &'a ProxyCandidate,
    pub(super) usage: TokenUsage,
    pub(super) amount: RequestBillingAmount,
}

pub(super) async fn enforce_preflight_access(state: &LlmProxyState, token: &ApiToken) -> Result<(), LlmProxyError> {
    ensure_token_quota(token)?;
    let Some(user) = user_for_token(state, token).await? else {
        return Ok(());
    };
    ensure_user_active(&user)?;
    ensure_wallet_quota(state, &user).await
}

pub(super) async fn settle_wallet_usage(state: &LlmProxyState, input: WalletSettlementInput<'_>) -> Result<(), LlmProxyError> {
    if input.amount.total_cost <= Decimal::ZERO {
        return Ok(());
    }
    let Some(token) = settlement_token(state, input.candidate).await? else {
        return Ok(());
    };
    let Some(user) = user_for_token(state, &token).await? else {
        return Ok(());
    };
    if user.quota_mode != USER_QUOTA_MODE_WALLET {
        return Ok(());
    }
    ensure_user_active(&user)?;
    settle_user_wallet(state, input, &token, &user).await
}

async fn user_for_token(state: &LlmProxyState, token: &ApiToken) -> Result<Option<CachedUserAccess>, LlmProxyError> {
    if token.token_type != ApiTokenType::User {
        return Ok(None);
    }
    let Some(user_id) = token.user_id.as_deref() else {
        return Err(disabled_user_error());
    };
    let snapshot = state.scheduling_snapshot().await?;
    snapshot
        .users
        .iter()
        .find(|user| user.id == user_id)
        .cloned()
        .map(Some)
        .ok_or_else(disabled_user_error)
}

fn ensure_user_active(user: &CachedUserAccess) -> Result<(), LlmProxyError> {
    if user.is_active {
        return Ok(());
    }
    Err(disabled_user_error())
}

fn ensure_token_quota(token: &ApiToken) -> Result<(), LlmProxyError> {
    if token.quota_limit.is_some_and(|limit| token.used_quota >= limit) {
        return Err(LlmProxyError::new_api_forbidden("pre-consume token quota failed", ERROR_CODE_TOKEN_QUOTA));
    }
    Ok(())
}

async fn ensure_wallet_quota(state: &LlmProxyState, user: &CachedUserAccess) -> Result<(), LlmProxyError> {
    if user.quota_mode != USER_QUOTA_MODE_WALLET && user.quota_mode != USER_QUOTA_MODE_UNLIMITED {
        return Err(LlmProxyError::Infrastructure(format!("unsupported user quota_mode: {}", user.quota_mode)));
    }
    if user.quota_mode == USER_QUOTA_MODE_UNLIMITED {
        return Ok(());
    }
    let wallet = WalletStore::new(state.database.clone())
        .find_by_user_id(&user.id)
        .await?
        .ok_or_else(wallet_quota_error)?;
    ensure_wallet_available(&wallet)
}

fn ensure_wallet_available(wallet: &Wallet) -> Result<(), LlmProxyError> {
    if wallet.status != WALLET_STATUS_ACTIVE {
        return Err(wallet_quota_error());
    }
    if wallet.limit_mode == WALLET_LIMIT_UNLIMITED {
        return Ok(());
    }
    if wallet.recharge_balance + wallet.gift_balance <= Decimal::ZERO {
        return Err(wallet_quota_error());
    }
    Ok(())
}

async fn settlement_token(state: &LlmProxyState, candidate: &ProxyCandidate) -> Result<Option<ApiToken>, LlmProxyError> {
    let Some(token_id) = candidate.trace.token_id.as_deref() else {
        return Ok(None);
    };
    ApiTokenStore::new(state.database.clone()).find_token(token_id).await.map_err(Into::into)
}

async fn settle_user_wallet(state: &LlmProxyState, input: WalletSettlementInput<'_>, token: &ApiToken, user: &CachedUserAccess) -> Result<(), LlmProxyError> {
    let wallet = WalletStore::new(state.database.clone())
        .find_by_user_id(&user.id)
        .await?
        .ok_or_else(wallet_quota_error)?;
    if wallet.limit_mode == WALLET_LIMIT_UNLIMITED {
        return Ok(());
    }
    ensure_wallet_available(&wallet)?;
    let description = snapshot::settlement_description(snapshot::DescriptionInput {
        input: &input,
        token,
        user,
        wallet: &wallet,
    })?;
    let consume = consume_input(&user.id, input.amount.total_cost, input.request_id, description);
    WalletStore::new(state.database.clone())
        .consume_with_transaction(consume)
        .await
        .map_err(wallet_settlement_error)?;
    Ok(())
}

fn disabled_user_error() -> LlmProxyError {
    LlmProxyError::new_api_forbidden("user is disabled or unavailable", ERROR_CODE_DISABLED_USER)
}

fn wallet_quota_error() -> LlmProxyError {
    LlmProxyError::new_api_forbidden("insufficient user quota", ERROR_CODE_WALLET_QUOTA)
}

fn wallet_settlement_error(error: StorageError) -> LlmProxyError {
    match error {
        StorageError::Conflict(message) if message == WALLET_CONSUME_INSUFFICIENT_BALANCE => wallet_quota_error(),
        error => error.into(),
    }
}

fn consume_input(user_id: &str, amount: Decimal, request_id: &str, description: String) -> WalletConsumeRecordInput {
    WalletConsumeRecordInput {
        user_id: user_id.into(),
        amount,
        category: CATEGORY_CONSUME.into(),
        reason_code: REASON_LLM_MODEL_USAGE.into(),
        link_type: Some(LINK_LLM_REQUEST_RECORD.into()),
        link_id: Some(request_id.into()),
        operator_id: None,
        description: Some(description),
    }
}
