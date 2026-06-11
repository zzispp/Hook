use async_trait::async_trait;
use rust_decimal::Decimal;
use time::format_description::well_known::{Iso8601, Rfc3339};
use types::{
    pagination::{Page, PageRequest},
    wallet::{
        AdminWalletConsumptionSummaryResponse, AdminWalletDailyUsageDetailsResponse, AdminWalletLedgerEntriesForWalletResponse,
        AdminWalletLedgerEntriesResponse, AdminWalletLedgerFilters, AdminWalletLedgerResponse, AdminWalletListFilters, AdminWalletListResponse,
        AdminWalletResponse, AdminWalletTransactionsResponse, Wallet, WalletAdjustment, WalletAdjustmentType, WalletBalanceResponse, WalletBalanceType,
        WalletDailyUsageDetailRequest, WalletDailyUsageDetailsResponse, WalletLedgerDateRange, WalletLedgerEntriesResponse, WalletLedgerEntry,
        WalletLedgerEntryFilters, WalletLedgerEntryResponse, WalletLedgerRangePreset, WalletRecharge, WalletSummaryResponse, WalletTransaction,
        WalletTransactionResponse, WalletTransactionsResponse,
    },
};

use crate::application::{NoSystemWalletProvider, SystemWalletProvider, WalletError, WalletRepository, WalletResult, WalletUseCase};

use super::validation::{validate_page, validate_positive_amount, validate_user_id, validate_wallet_id};

mod system_wallet;

const CATEGORY_ADJUST: &str = "adjust";
const CATEGORY_RECHARGE: &str = "recharge";
const REASON_ADJUST_ADMIN: &str = "adjust_admin";
const REASON_TOPUP_ADMIN_MANUAL: &str = "topup_admin_manual";
const LINK_ADMIN_ACTION: &str = "admin_action";

pub struct WalletService<R, S = NoSystemWalletProvider> {
    repository: R,
    system_wallets: S,
}

impl<R> WalletService<R, NoSystemWalletProvider>
where
    R: WalletRepository,
{
    pub const fn new(repository: R) -> Self {
        Self {
            repository,
            system_wallets: NoSystemWalletProvider,
        }
    }
}

impl<R, S> WalletService<R, S>
where
    R: WalletRepository,
    S: SystemWalletProvider,
{
    pub const fn with_system_wallet(repository: R, system_wallets: S) -> Self {
        Self { repository, system_wallets }
    }

    async fn user_wallet(&self, user_id: &str) -> WalletResult<Wallet> {
        validate_user_id(user_id)?;
        if let Some(wallet) = system_wallet::wallet_for_user(self.system_wallets.system_wallet(), user_id) {
            return Ok(wallet);
        }
        match self.repository.find_by_user_id(user_id).await? {
            Some(wallet) => Ok(wallet),
            None => self.repository.ensure_user_wallet(user_id).await,
        }
    }

    async fn wallet_by_id(&self, wallet_id: &str) -> WalletResult<Wallet> {
        validate_wallet_id(wallet_id)?;
        if system_wallet::wallet_for_id(self.system_wallets.system_wallet(), wallet_id).is_some() {
            return Err(WalletError::Forbidden);
        }
        self.repository.find_by_id(wallet_id).await?.ok_or(WalletError::NotFound)
    }

    fn system_admin_wallet(&self, filters: &AdminWalletListFilters) -> Option<AdminWalletResponse> {
        self.system_wallets
            .system_wallet()
            .filter(|record| system_wallet::system_wallet_matches(record, filters))
            .map(system_wallet::admin_wallet_response)
    }
}

#[async_trait]
impl<R, S> WalletUseCase for WalletService<R, S>
where
    R: WalletRepository,
    S: SystemWalletProvider,
{
    async fn balance(&self, user_id: &str) -> WalletResult<WalletBalanceResponse> {
        let wallet = self.user_wallet(user_id).await?;
        Ok(WalletBalanceResponse::from(WalletSummaryResponse::from(wallet)))
    }

    async fn admin_balance(&self, user_id: &str) -> WalletResult<WalletBalanceResponse> {
        let wallet = self.user_wallet(user_id).await?;
        Ok(WalletBalanceResponse::from(WalletSummaryResponse::from(wallet)))
    }

    async fn transactions(&self, user_id: &str, page: PageRequest) -> WalletResult<WalletTransactionsResponse> {
        validate_page(page)?;
        if let Some(wallet) = system_wallet::wallet_for_user(self.system_wallets.system_wallet(), user_id) {
            return Ok(system_wallet::user_transactions_response(wallet, page));
        }
        let wallet = self.user_wallet(user_id).await?;
        let wallet_summary = WalletSummaryResponse::from(wallet.clone());
        let transactions = self.repository.page_transactions(&wallet.id.0, page).await?;
        Ok(user_transactions_response(wallet_summary, transactions))
    }

    async fn ledger_entries(
        &self,
        user_id: &str,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<WalletLedgerEntriesResponse> {
        validate_page(page)?;
        let filters = validated_ledger_filters(filters, tz_offset_minutes)?;
        let wallet = self.user_wallet(user_id).await?;
        let entries = self.repository.page_ledger_entries(&wallet.id.0, page, filters, tz_offset_minutes).await?;
        Ok(ledger_entries_response(WalletSummaryResponse::from(wallet), entries))
    }

    async fn daily_usage_transactions(
        &self,
        user_id: &str,
        page: PageRequest,
        request: WalletDailyUsageDetailRequest,
    ) -> WalletResult<WalletDailyUsageDetailsResponse> {
        validate_page(page)?;
        validate_daily_usage_request(&request)?;
        let wallet = self.user_wallet(user_id).await?;
        let transactions = self.repository.page_daily_usage_transactions(&wallet.id.0, page, request).await?;
        Ok(daily_usage_details_response(transactions))
    }

    async fn admin_wallets(&self, page: PageRequest, filters: AdminWalletListFilters) -> WalletResult<AdminWalletListResponse> {
        validate_page(page)?;
        let system_wallet = self.system_admin_wallet(&filters);
        let (slice, include_system, db_limit) = system_wallet::admin_wallet_slice(page, system_wallet.is_some());
        let wallets = self.repository.page_admin_wallets(slice, filters).await?;
        let wallets = system_wallet::admin_wallet_page(page, wallets, system_wallet, include_system, db_limit);
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

    async fn admin_ledger_entries(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletLedgerEntriesResponse> {
        validate_page(page)?;
        let filters = validated_ledger_filters(filters, tz_offset_minutes)?;
        let entries = self.repository.page_admin_ledger_entries(page, filters, tz_offset_minutes).await?;
        Ok(AdminWalletLedgerEntriesResponse {
            items: entries.items,
            total: entries.total,
            page: entries.page,
            page_size: entries.page_size,
        })
    }

    async fn admin_consumption_summary(
        &self,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletConsumptionSummaryResponse> {
        validate_page(page)?;
        let filters = validated_ledger_filters(filters, tz_offset_minutes)?;
        let summary = self.repository.page_admin_consumption_summary(page, filters).await?;
        Ok(AdminWalletConsumptionSummaryResponse {
            items: summary.items,
            total: summary.total,
            page: summary.page,
            page_size: summary.page_size,
        })
    }

    async fn admin_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<AdminWalletTransactionsResponse> {
        validate_page(page)?;
        validate_wallet_id(wallet_id)?;
        if let Some(record) = system_wallet::wallet_for_id(self.system_wallets.system_wallet(), wallet_id) {
            return Ok(system_wallet::admin_transactions_response(record, page));
        }
        let wallet = self.repository.find_admin_wallet_by_id(wallet_id).await?.ok_or(WalletError::NotFound)?;
        let transactions = self.repository.page_transactions(wallet_id, page).await?;
        Ok(admin_transactions_response(wallet, transactions))
    }

    async fn admin_ledger_entries_for_wallet(
        &self,
        wallet_id: &str,
        page: PageRequest,
        filters: WalletLedgerEntryFilters,
        tz_offset_minutes: i32,
    ) -> WalletResult<AdminWalletLedgerEntriesForWalletResponse> {
        validate_page(page)?;
        validate_wallet_id(wallet_id)?;
        let filters = validated_ledger_filters(filters, tz_offset_minutes)?;
        let wallet = self.repository.find_admin_wallet_by_id(wallet_id).await?.ok_or(WalletError::NotFound)?;
        let entries = self.repository.page_ledger_entries(wallet_id, page, filters, tz_offset_minutes).await?;
        Ok(admin_ledger_entries_for_wallet_response(wallet, entries))
    }

    async fn admin_daily_usage_transactions(
        &self,
        wallet_id: &str,
        page: PageRequest,
        request: WalletDailyUsageDetailRequest,
    ) -> WalletResult<AdminWalletDailyUsageDetailsResponse> {
        validate_page(page)?;
        validate_wallet_id(wallet_id)?;
        validate_daily_usage_request(&request)?;
        let wallet = self.repository.find_admin_wallet_by_id(wallet_id).await?.ok_or(WalletError::NotFound)?;
        let transactions = self.repository.page_daily_usage_transactions(wallet_id, page, request).await?;
        Ok(admin_daily_usage_details_response(wallet, transactions))
    }

    async fn adjust_wallet(&self, input: WalletAdjustment) -> WalletResult<WalletTransaction> {
        validate_positive_amount("adjust amount", input.amount)?;
        let wallet = self.wallet_by_id(&input.wallet_id).await?;
        let (updated_wallet, transaction) = apply_adjustment(wallet, input)?;
        self.repository.save_ledger_entry(updated_wallet, transaction).await
    }

    async fn recharge_wallet(&self, input: WalletRecharge) -> WalletResult<WalletTransaction> {
        validate_positive_amount("recharge amount", input.amount)?;
        let wallet = self.wallet_by_id(&input.wallet_id).await?;
        let (updated_wallet, transaction) = apply_recharge(wallet, input);
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

fn ledger_entries_response(wallet: WalletSummaryResponse, page: Page<WalletLedgerEntry>) -> WalletLedgerEntriesResponse {
    WalletLedgerEntriesResponse {
        wallet,
        items: ledger_entry_responses(page.items),
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

fn admin_ledger_entries_for_wallet_response(wallet: AdminWalletResponse, page: Page<WalletLedgerEntry>) -> AdminWalletLedgerEntriesForWalletResponse {
    AdminWalletLedgerEntriesForWalletResponse {
        wallet,
        items: ledger_entry_responses(page.items),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn daily_usage_details_response(page: Page<WalletTransaction>) -> WalletDailyUsageDetailsResponse {
    WalletDailyUsageDetailsResponse {
        items: transaction_responses(page.items),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn admin_daily_usage_details_response(wallet: AdminWalletResponse, page: Page<WalletTransaction>) -> AdminWalletDailyUsageDetailsResponse {
    AdminWalletDailyUsageDetailsResponse {
        wallet,
        items: transaction_responses(page.items),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn ledger_entry_responses(items: Vec<WalletLedgerEntry>) -> Vec<WalletLedgerEntryResponse> {
    items.into_iter().map(WalletLedgerEntryResponse::from).collect()
}

fn transaction_responses(items: Vec<WalletTransaction>) -> Vec<WalletTransactionResponse> {
    items.into_iter().map(WalletTransactionResponse::from).collect()
}

fn validate_tz_offset(value: i32) -> WalletResult<()> {
    if !(-1_080..=1_080).contains(&value) {
        return Err(WalletError::InvalidInput("tz_offset_minutes must be between -1080 and 1080".into()));
    }
    Ok(())
}

fn validate_daily_usage_request(request: &WalletDailyUsageDetailRequest) -> WalletResult<()> {
    validate_tz_offset(request.tz_offset_minutes)?;
    if is_iso_date(&request.local_date) {
        return Ok(());
    }
    Err(WalletError::InvalidInput("date must use YYYY-MM-DD".into()))
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit)
}

fn validated_ledger_filters(filters: WalletLedgerEntryFilters, tz_offset_minutes: i32) -> WalletResult<WalletLedgerEntryFilters> {
    validate_tz_offset(tz_offset_minutes)?;
    let date_range = ledger_date_range(&filters, tz_offset_minutes)?;
    Ok(WalletLedgerEntryFilters { date_range, ..filters })
}

fn ledger_date_range(filters: &WalletLedgerEntryFilters, tz_offset_minutes: i32) -> WalletResult<Option<WalletLedgerDateRange>> {
    match filters.range_preset {
        WalletLedgerRangePreset::All => Ok(None),
        WalletLedgerRangePreset::Today => preset_ledger_date_range(1, tz_offset_minutes),
        WalletLedgerRangePreset::Last7Days => preset_ledger_date_range(7, tz_offset_minutes),
        WalletLedgerRangePreset::Last30Days => preset_ledger_date_range(30, tz_offset_minutes),
        WalletLedgerRangePreset::Custom => custom_ledger_date_range(filters, tz_offset_minutes),
    }
}

fn preset_ledger_date_range(days: i64, tz_offset_minutes: i32) -> WalletResult<Option<WalletLedgerDateRange>> {
    let today = current_local_date(tz_offset_minutes)?;
    let start = today - time::Duration::days(days - 1);
    date_range_from_dates(start, today, tz_offset_minutes).map(Some)
}

fn custom_ledger_date_range(filters: &WalletLedgerEntryFilters, tz_offset_minutes: i32) -> WalletResult<Option<WalletLedgerDateRange>> {
    let start = required_date(filters.start_date.as_deref(), "start_date")?;
    let end = required_date(filters.end_date.as_deref(), "end_date")?;
    date_range_from_dates(start, end, tz_offset_minutes).map(Some)
}

fn required_date(value: Option<&str>, field: &str) -> WalletResult<time::Date> {
    parse_ledger_date(
        value.ok_or_else(|| WalletError::InvalidInput(format!("{field} is required for custom range")))?,
        field,
    )
}

fn date_range_from_dates(start: time::Date, end: time::Date, tz_offset_minutes: i32) -> WalletResult<WalletLedgerDateRange> {
    if start > end {
        return Err(WalletError::InvalidInput("start_date must be before or equal to end_date".into()));
    }
    let next_day = end.next_day().ok_or_else(|| WalletError::InvalidInput("end_date is out of range".into()))?;
    Ok(WalletLedgerDateRange {
        start_date: start.to_string(),
        end_date: end.to_string(),
        started_at: format_timestamp(local_date_start_utc(start, tz_offset_minutes)?)?,
        ended_at: format_timestamp(local_date_start_utc(next_day, tz_offset_minutes)?)?,
    })
}

fn current_local_date(tz_offset_minutes: i32) -> WalletResult<time::Date> {
    Ok(time::OffsetDateTime::now_utc().to_offset(utc_offset(tz_offset_minutes)?).date())
}

fn local_date_start_utc(date: time::Date, tz_offset_minutes: i32) -> WalletResult<time::OffsetDateTime> {
    let offset = utc_offset(tz_offset_minutes)?;
    date.with_hms(0, 0, 0)
        .map(|value| value.assume_offset(offset).to_offset(time::UtcOffset::UTC))
        .map_err(|error| WalletError::InvalidInput(format!("invalid local date boundary: {error}")))
}

fn utc_offset(tz_offset_minutes: i32) -> WalletResult<time::UtcOffset> {
    let seconds = tz_offset_minutes
        .checked_mul(60)
        .ok_or_else(|| WalletError::InvalidInput("tz_offset_minutes exceeds supported range".into()))?;
    time::UtcOffset::from_whole_seconds(seconds).map_err(|_| WalletError::InvalidInput("tz_offset_minutes must be between -1080 and 1080".into()))
}

fn parse_ledger_date(value: &str, field: &str) -> WalletResult<time::Date> {
    time::Date::parse(value, &Iso8601::DEFAULT).map_err(|error| WalletError::InvalidInput(format!("{field} must use YYYY-MM-DD: {error}")))
}

fn format_timestamp(value: time::OffsetDateTime) -> WalletResult<String> {
    value
        .format(&Rfc3339)
        .map_err(|error| WalletError::InvalidInput(format!("invalid range timestamp: {error}")))
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

fn apply_recharge(wallet: Wallet, input: WalletRecharge) -> (Wallet, WalletTransaction) {
    let before_recharge = wallet.recharge_balance;
    let before_gift = wallet.gift_balance;
    let before_total = before_recharge + before_gift;
    let after_recharge = before_recharge + input.amount;
    let after_total = after_recharge + before_gift;
    let updated_wallet = recharged_wallet(wallet, after_recharge, input.amount);
    let transaction = recharge_transaction(
        &updated_wallet,
        input,
        BalanceSnapshot {
            before_recharge,
            after_recharge,
            before_gift,
            after_gift: before_gift,
            before_total,
            after_total,
        },
    );
    (updated_wallet, transaction)
}

fn recharged_wallet(wallet: Wallet, recharge_balance: Decimal, amount: Decimal) -> Wallet {
    Wallet {
        recharge_balance,
        total_recharged: wallet.total_recharged + amount,
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

fn recharge_transaction(wallet: &Wallet, input: WalletRecharge, snapshot: BalanceSnapshot) -> WalletTransaction {
    WalletTransaction {
        id: String::new(),
        wallet_id: wallet.id.0.clone(),
        category: CATEGORY_RECHARGE.into(),
        reason_code: REASON_TOPUP_ADMIN_MANUAL.into(),
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
