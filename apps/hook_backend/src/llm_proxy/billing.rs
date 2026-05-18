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

use super::{
    LlmProxyError, LlmProxyState,
    audit::TokenUsage,
    cache::snapshot::{CachedUserAccess, SchedulingSnapshot},
    candidate::ProxyCandidate,
};

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
    let snapshot = state.scheduling_snapshot().await?;
    billing_user_for_token(&snapshot, token)
}

fn billing_user_for_token(snapshot: &SchedulingSnapshot, token: &ApiToken) -> Result<Option<CachedUserAccess>, LlmProxyError> {
    let user_id = match token.token_type {
        ApiTokenType::User | ApiTokenType::Independent => token.user_id.as_deref().ok_or_else(disabled_user_error)?,
    };
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
    let wallet = wallet_for_user(state, &user.id).await?;
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
    let wallet = wallet_for_user(state, &user.id).await?;
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

async fn wallet_for_user(state: &LlmProxyState, user_id: &str) -> Result<Wallet, LlmProxyError> {
    if let Some(wallet) = state.system_wallet_for_user(user_id) {
        return Ok(wallet);
    }
    WalletStore::new(state.database.clone())
        .find_by_user_id(user_id)
        .await?
        .ok_or_else(wallet_quota_error)
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

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use types::{
        api_token::{ApiToken, ApiTokenType, ModelAccessMode},
        provider::ProviderSchedulingMode,
        system_setting::RequestRecordLevel,
    };

    use super::billing_user_for_token;
    use crate::llm_proxy::cache::snapshot::{CachedUserAccess, SchedulingSnapshot};

    #[test]
    fn billing_user_for_token_returns_independent_token_owner() {
        let snapshot = snapshot_with_user(user_access("system-admin", "admin", "wallet"));
        let token = api_token(ApiTokenType::Independent, Some("system-admin"));

        let user = billing_user_for_token(&snapshot, &token).unwrap();

        assert_eq!(user.as_ref().map(|user| user.id.as_str()), Some("system-admin"));
    }

    fn snapshot_with_user(user: CachedUserAccess) -> SchedulingSnapshot {
        SchedulingSnapshot {
            default_rate_limit_rpm: 0,
            scheduling_mode: ProviderSchedulingMode::FixedOrder,
            client_request_record_level: RequestRecordLevel::Basic,
            client_max_request_body_size_kb: 1024,
            client_max_response_body_size_kb: 1024,
            client_sensitive_request_headers: String::new(),
            provider_request_record_level: RequestRecordLevel::Basic,
            provider_max_request_body_size_kb: 1024,
            provider_max_response_body_size_kb: 1024,
            provider_sensitive_request_headers: String::new(),
            provider_cooldown_policy: Default::default(),
            models: Vec::new(),
            groups: Vec::new(),
            users: vec![user],
            providers: Vec::new(),
        }
    }

    fn user_access(id: &str, username: &str, quota_mode: &str) -> CachedUserAccess {
        CachedUserAccess {
            id: id.into(),
            username: username.into(),
            is_active: true,
            allowed_model_ids: Vec::new(),
            allowed_provider_ids: Vec::new(),
            quota_mode: quota_mode.into(),
            rate_limit_rpm: None,
        }
    }

    fn api_token(token_type: ApiTokenType, user_id: Option<&str>) -> ApiToken {
        ApiToken {
            id: "token-a".into(),
            user_id: user_id.map(str::to_owned),
            token_type,
            name: "Token A".into(),
            token_value: String::new(),
            token_hash: String::new(),
            token_prefix: "sk-test".into(),
            group_code: "default".into(),
            expires_at: None,
            model_access_mode: ModelAccessMode::All,
            allowed_model_ids: Vec::new(),
            rate_limit_rpm: None,
            quota_limit: None,
            used_quota: Decimal::ZERO,
            request_count: 0,
            is_active: true,
            last_used_at: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}
