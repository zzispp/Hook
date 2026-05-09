use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::PageRequest,
    wallet::{
        Wallet, WalletAdjustment, WalletBalanceResponse, WalletBalanceType, WalletSummaryResponse, WalletTransaction, WalletTransactionResponse,
        WalletTransactionsResponse,
    },
};

use crate::application::{WalletError, WalletRepository, WalletResult, WalletUseCase};

use super::validation::{validate_adjust_amount, validate_page, validate_user_id, validate_wallet_id};

const CATEGORY_ADJUST: &str = "adjust";
const REASON_ADJUST_ADMIN: &str = "adjust_admin";
const LINK_ADMIN_ACTION: &str = "admin_action";

pub struct WalletService<R> {
    repository: R,
}

impl<R> WalletService<R>
where
    R: WalletRepository,
{
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }

    async fn user_wallet(&self, user_id: &str) -> WalletResult<Wallet> {
        validate_user_id(user_id)?;
        match self.repository.find_by_user_id(user_id).await? {
            Some(wallet) => Ok(wallet),
            None => self.repository.create_user_wallet(user_id).await,
        }
    }

    async fn wallet_by_id(&self, wallet_id: &str) -> WalletResult<Wallet> {
        validate_wallet_id(wallet_id)?;
        self.repository.find_by_id(wallet_id).await?.ok_or(WalletError::NotFound)
    }
}

#[async_trait]
impl<R> WalletUseCase for WalletService<R>
where
    R: WalletRepository,
{
    async fn balance(&self, user_id: &str) -> WalletResult<WalletBalanceResponse> {
        let wallet = self.user_wallet(user_id).await?;
        Ok(WalletBalanceResponse::from(WalletSummaryResponse::from(wallet)))
    }

    async fn transactions(&self, user_id: &str, page: PageRequest) -> WalletResult<WalletTransactionsResponse> {
        validate_page(page)?;
        let wallet = self.user_wallet(user_id).await?;
        let wallet_summary = WalletSummaryResponse::from(wallet.clone());
        let page = self.repository.page_transactions(&wallet.id.0, page).await?;
        Ok(WalletTransactionsResponse {
            wallet: wallet_summary,
            items: page.items.into_iter().map(WalletTransactionResponse::from).collect(),
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        })
    }

    async fn adjust_wallet(&self, input: WalletAdjustment) -> WalletResult<WalletTransaction> {
        validate_adjust_amount(input.amount)?;
        let wallet = self.wallet_by_id(&input.wallet_id).await?;
        let (updated_wallet, transaction) = apply_adjustment(wallet, input)?;
        self.repository.save_ledger_entry(updated_wallet, transaction).await
    }
}

fn apply_adjustment(wallet: Wallet, input: WalletAdjustment) -> WalletResult<(Wallet, WalletTransaction)> {
    let before_recharge = wallet.recharge_balance;
    let before_gift = wallet.gift_balance;
    let before_total = before_recharge + before_gift;
    let after_recharge = adjusted_recharge_balance(&input, before_recharge);
    let after_gift = adjusted_gift_balance(&input, before_gift)?;
    let after_total = after_recharge + after_gift;
    let updated_wallet = adjusted_wallet(wallet, &input, after_recharge, after_gift);
    let transaction = adjustment_transaction(
        &updated_wallet,
        input,
        BalanceSnapshot {
            before_recharge,
            after_recharge,
            before_gift,
            after_gift,
            before_total,
            after_total,
        },
    );
    Ok((updated_wallet, transaction))
}

fn adjusted_recharge_balance(input: &WalletAdjustment, before: Decimal) -> Decimal {
    match input.balance_type {
        WalletBalanceType::Recharge => before + input.amount,
        WalletBalanceType::Gift => before,
    }
}

fn adjusted_gift_balance(input: &WalletAdjustment, before: Decimal) -> WalletResult<Decimal> {
    match input.balance_type {
        WalletBalanceType::Recharge => Ok(before),
        WalletBalanceType::Gift => {
            let after = before + input.amount;
            if after < Decimal::ZERO {
                return Err(WalletError::InvalidInput("gift balance cannot be negative".into()));
            }
            Ok(after)
        }
    }
}

fn adjusted_wallet(wallet: Wallet, input: &WalletAdjustment, recharge_balance: Decimal, gift_balance: Decimal) -> Wallet {
    Wallet {
        recharge_balance,
        gift_balance,
        total_adjusted: wallet.total_adjusted + input.amount,
        ..wallet
    }
}

fn adjustment_transaction(wallet: &Wallet, input: WalletAdjustment, snapshot: BalanceSnapshot) -> WalletTransaction {
    WalletTransaction {
        id: String::new(),
        wallet_id: wallet.id.0.clone(),
        category: CATEGORY_ADJUST.into(),
        reason_code: REASON_ADJUST_ADMIN.into(),
        amount: input.amount,
        balance_before: snapshot.before_total,
        balance_after: snapshot.after_total,
        recharge_balance_before: snapshot.before_recharge,
        recharge_balance_after: snapshot.after_recharge,
        gift_balance_before: snapshot.before_gift,
        gift_balance_after: snapshot.after_gift,
        link_type: Some(LINK_ADMIN_ACTION.into()),
        link_id: Some(wallet.id.0.clone()),
        operator_id: input.operator_id,
        description: input.description,
        created_at: String::new(),
    }
}

struct BalanceSnapshot {
    before_recharge: Decimal,
    after_recharge: Decimal,
    before_gift: Decimal,
    after_gift: Decimal,
    before_total: Decimal,
    after_total: Decimal,
}

#[cfg(test)]
mod tests;
