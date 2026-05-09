use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest},
    wallet::{
        AdminWalletLedgerFilters, AdminWalletLedgerResponse, AdminWalletListFilters, AdminWalletListResponse, AdminWalletResponse, AdminWalletTransactionsResponse,
        Wallet, WalletAdjustment, WalletAdjustmentType, WalletBalanceResponse, WalletBalanceType, WalletSummaryResponse, WalletTransaction, WalletTransactionResponse,
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
            None => self.repository.ensure_user_wallet(user_id).await,
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
        let transactions = self.repository.page_transactions(&wallet.id.0, page).await?;
        Ok(user_transactions_response(wallet_summary, transactions))
    }

    async fn admin_wallets(&self, page: PageRequest, filters: AdminWalletListFilters) -> WalletResult<AdminWalletListResponse> {
        validate_page(page)?;
        let wallets = self.repository.page_admin_wallets(page, filters).await?;
        Ok(AdminWalletListResponse {
            items: wallets.items,
            total: wallets.total,
            page: wallets.page,
            page_size: wallets.page_size,
        })
    }

    async fn admin_ledger(&self, page: PageRequest, filters: AdminWalletLedgerFilters) -> WalletResult<AdminWalletLedgerResponse> {
        validate_page(page)?;
        let ledger = self.repository.page_admin_ledger(page, filters).await?;
        Ok(AdminWalletLedgerResponse {
            items: ledger.items,
            total: ledger.total,
            page: ledger.page,
            page_size: ledger.page_size,
        })
    }

    async fn admin_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<AdminWalletTransactionsResponse> {
        validate_page(page)?;
        validate_wallet_id(wallet_id)?;
        let wallet = self.repository.find_admin_wallet_by_id(wallet_id).await?.ok_or(WalletError::NotFound)?;
        let transactions = self.repository.page_transactions(wallet_id, page).await?;
        Ok(admin_transactions_response(wallet, transactions))
    }

    async fn adjust_wallet(&self, input: WalletAdjustment) -> WalletResult<WalletTransaction> {
        validate_adjust_amount(input.amount)?;
        let wallet = self.wallet_by_id(&input.wallet_id).await?;
        let (updated_wallet, transaction) = apply_adjustment(wallet, input)?;
        self.repository.save_ledger_entry(updated_wallet, transaction).await
    }
}

fn user_transactions_response(wallet: WalletSummaryResponse, page: Page<WalletTransaction>) -> WalletTransactionsResponse {
    WalletTransactionsResponse {
        wallet,
        items: transaction_responses(page.items),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn admin_transactions_response(wallet: AdminWalletResponse, page: Page<WalletTransaction>) -> AdminWalletTransactionsResponse {
    AdminWalletTransactionsResponse {
        wallet,
        items: transaction_responses(page.items),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn transaction_responses(items: Vec<WalletTransaction>) -> Vec<WalletTransactionResponse> {
    items.into_iter().map(WalletTransactionResponse::from).collect()
}

fn apply_adjustment(wallet: Wallet, input: WalletAdjustment) -> WalletResult<(Wallet, WalletTransaction)> {
    let before_recharge = wallet.recharge_balance;
    let before_gift = wallet.gift_balance;
    let before_total = before_recharge + before_gift;
    let signed_amount = signed_adjust_amount(&input);
    let after_recharge = adjusted_recharge_balance(&input, before_recharge, signed_amount)?;
    let after_gift = adjusted_gift_balance(&input, before_gift, signed_amount)?;
    let after_total = after_recharge + after_gift;
    let updated_wallet = adjusted_wallet(wallet, after_recharge, after_gift, signed_amount);
    let transaction = adjustment_transaction(
        &updated_wallet,
        input,
        signed_amount,
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

fn signed_adjust_amount(input: &WalletAdjustment) -> Decimal {
    match input.adjustment_type {
        WalletAdjustmentType::Increase => input.amount,
        WalletAdjustmentType::Deduct => -input.amount,
    }
}

fn adjusted_recharge_balance(input: &WalletAdjustment, before: Decimal, amount: Decimal) -> WalletResult<Decimal> {
    match input.balance_type {
        WalletBalanceType::Recharge => non_negative_balance(before + amount, "recharge balance"),
        WalletBalanceType::Gift => Ok(before),
    }
}

fn adjusted_gift_balance(input: &WalletAdjustment, before: Decimal, amount: Decimal) -> WalletResult<Decimal> {
    match input.balance_type {
        WalletBalanceType::Recharge => Ok(before),
        WalletBalanceType::Gift => non_negative_balance(before + amount, "gift balance"),
    }
}

fn non_negative_balance(value: Decimal, field: &str) -> WalletResult<Decimal> {
    if value < Decimal::ZERO {
        return Err(WalletError::InvalidInput(format!("{field} cannot be negative")));
    }
    Ok(value)
}

fn adjusted_wallet(wallet: Wallet, recharge_balance: Decimal, gift_balance: Decimal, amount: Decimal) -> Wallet {
    Wallet {
        recharge_balance,
        gift_balance,
        total_adjusted: wallet.total_adjusted + amount,
        ..wallet
    }
}

fn adjustment_transaction(wallet: &Wallet, input: WalletAdjustment, amount: Decimal, snapshot: BalanceSnapshot) -> WalletTransaction {
    WalletTransaction {
        id: String::new(),
        wallet_id: wallet.id.0.clone(),
        category: CATEGORY_ADJUST.into(),
        reason_code: REASON_ADJUST_ADMIN.into(),
        amount,
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
