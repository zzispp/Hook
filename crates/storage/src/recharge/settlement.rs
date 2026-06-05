use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QuerySelect, Set, TransactionTrait};
use types::recharge::{RECHARGE_ORDER_STATUS_PAID, RECHARGE_ORDER_STATUS_PENDING};

use super::{
    RechargeOrderRecord, RechargePaymentSettlementInput, RechargePaymentSettlementRecord, RechargeStore,
    record::{affiliate_commissions as affiliate_commission_records, recharge_orders as recharge_order_records},
};
use crate::{
    Database, StorageError, StorageResult, json,
    setting::{SYSTEM_SETTINGS_ID, system_setting_records},
    user::{UserEntity, UserRecord},
    wallet::{wallet_records, wallet_records::ActiveModel as WalletActiveModel, wallet_transaction_records},
};

const DEFAULT_WALLET_STATUS: &str = "active";
const DEFAULT_WALLET_LIMIT_MODE: &str = "finite";
const PAYMENT_LINK_TYPE: &str = "payment_order";
const AFFILIATE_LINK_TYPE: &str = "affiliate_commission";
const TOPUP_GATEWAY_REASON: &str = "topup_gateway";
const AFFILIATE_COMMISSION_REASON: &str = "affiliate_commission";
const AFFILIATE_COMMISSION_STATUS_SUCCESS: &str = "success";
const AFFILIATE_COMMISSION_STATUS_FAILED: &str = "failed";
const AFFILIATE_COMMISSION_FAILURE_BELOW_MINIMUM: &str = "below_min_commission_amount";
const RECHARGE_CATEGORY: &str = "recharge";
const GIFT_CATEGORY: &str = "gift";
const PERCENT_DENOMINATOR: i64 = 100;

impl RechargeStore {
    pub async fn settle_paid_order(&self, input: RechargePaymentSettlementInput) -> StorageResult<RechargePaymentSettlementRecord> {
        let tx = self.database.connection().begin().await?;
        let now = time::OffsetDateTime::now_utc();
        let record = lock_order_by_no(&input.order_no, &tx).await?.ok_or(StorageError::NotFound)?;
        if record.status == RECHARGE_ORDER_STATUS_PAID {
            let order = record.into_response(String::new(), String::new());
            tx.commit().await?;
            return Ok(RechargePaymentSettlementRecord { order, settled: false });
        }
        if record.status != RECHARGE_ORDER_STATUS_PENDING {
            return Err(StorageError::Conflict(format!("recharge order is {}", record.status)));
        }
        ensure_payment_channel_matches(&record, &input.payment_channel_code)?;
        ensure_provider_trade_no_present(input.provider_trade_no.as_deref())?;
        ensure_payable_amount_matches(&record, input.payable_amount)?;
        ensure_provider_trade_no_unused(&record, input.provider_trade_no.as_deref(), &tx).await?;
        let wallet = ensure_wallet_in_tx(&self.database, &record.user_id, &tx).await?;
        credit_wallet_and_insert_transaction(&self.database, &wallet, &record, now, &tx).await?;
        settle_affiliate_commission(&self.database, &record, now, &tx).await?;
        let order = mark_order_paid(record, input, now, &tx).await?;
        tx.commit().await?;
        Ok(RechargePaymentSettlementRecord {
            order: order.into_response(String::new(), String::new()),
            settled: true,
        })
    }
}

fn ensure_payment_channel_matches(record: &RechargeOrderRecord, channel_code: &str) -> StorageResult<()> {
    if record.payment_channel_code.as_deref() == Some(channel_code) {
        return Ok(());
    }
    Err(StorageError::Conflict("payment channel mismatch".into()))
}

fn ensure_provider_trade_no_present(provider_trade_no: Option<&str>) -> StorageResult<()> {
    if provider_trade_no.is_some_and(|value| !value.trim().is_empty()) {
        return Ok(());
    }
    Err(StorageError::Conflict("provider trade number is required".into()))
}

fn ensure_payable_amount_matches(record: &RechargeOrderRecord, payable_amount: Option<Decimal>) -> StorageResult<()> {
    let Some(payable_amount) = payable_amount else {
        return Err(StorageError::Conflict("payment amount is required".into()));
    };
    if record.payable_amount == payable_amount {
        return Ok(());
    }
    Err(StorageError::Conflict("payment amount mismatch".into()))
}

async fn ensure_provider_trade_no_unused(
    record: &RechargeOrderRecord,
    provider_trade_no: Option<&str>,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    let Some(provider_trade_no) = provider_trade_no.filter(|value| !value.trim().is_empty()) else {
        return Ok(());
    };
    let Some(channel_code) = record.payment_channel_code.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(());
    };
    let existing = recharge_order_records::Entity::find()
        .filter(recharge_order_records::Column::PaymentChannelCode.eq(channel_code))
        .filter(recharge_order_records::Column::ProviderTradeNo.eq(provider_trade_no))
        .filter(recharge_order_records::Column::OrderNo.ne(record.order_no.clone()))
        .lock_exclusive()
        .one(tx)
        .await?;
    if existing.is_some() {
        return Err(StorageError::Conflict("provider trade number has already been settled".into()));
    }
    Ok(())
}

async fn lock_order_by_no(order_no: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<RechargeOrderRecord>> {
    recharge_order_records::Entity::find()
        .filter(recharge_order_records::Column::OrderNo.eq(order_no))
        .lock_exclusive()
        .one(tx)
        .await
        .map_err(StorageError::from)
}

async fn mark_order_paid(
    record: RechargeOrderRecord,
    input: RechargePaymentSettlementInput,
    now: time::OffsetDateTime,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<RechargeOrderRecord> {
    let mut active: recharge_order_records::ActiveModel = record.into();
    active.status = Set(RECHARGE_ORDER_STATUS_PAID.into());
    active.provider_trade_no = Set(input.provider_trade_no);
    active.payment_method = Set(Some(input.payment_method));
    active.payment_request_json = Set(Some(json::encode_required(&input.callback_payload)?));
    active.paid_at = Set(Some(now));
    active.updated_at = Set(now);
    active.update(tx).await.map_err(StorageError::from)
}

async fn ensure_wallet_in_tx(database: &Database, user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<types::wallet::Wallet> {
    if let Some(wallet) = wallet_by_user_in_tx(user_id, tx).await? {
        return Ok(wallet);
    }
    insert_wallet_in_tx(database, user_id, tx).await?;
    wallet_by_user_in_tx(user_id, tx).await?.ok_or(StorageError::NotFound)
}

async fn wallet_by_user_in_tx(user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<types::wallet::Wallet>> {
    wallet_records::Entity::find()
        .filter(wallet_records::Column::UserId.eq(user_id))
        .lock_exclusive()
        .one(tx)
        .await
        .map(|record| record.map(types::wallet::Wallet::from))
        .map_err(StorageError::from)
}

async fn insert_wallet_in_tx(database: &Database, user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    wallet_records::Entity::insert(wallet_active_model(database.next_id(), user_id, now))
        .on_conflict_do_nothing_on([wallet_records::Column::UserId])
        .exec_without_returning(tx)
        .await?;
    Ok(())
}

fn wallet_active_model(id: String, user_id: &str, now: time::OffsetDateTime) -> WalletActiveModel {
    WalletActiveModel {
        id: Set(id),
        user_id: Set(user_id.to_owned()),
        recharge_balance: Set(Decimal::ZERO),
        gift_balance: Set(Decimal::ZERO),
        currency: Set(currency::ACCOUNTING_CURRENCY.into()),
        status: Set(DEFAULT_WALLET_STATUS.into()),
        limit_mode: Set(DEFAULT_WALLET_LIMIT_MODE.into()),
        total_recharged: Set(Decimal::ZERO),
        total_consumed: Set(Decimal::ZERO),
        total_refunded: Set(Decimal::ZERO),
        total_adjusted: Set(Decimal::ZERO),
        created_at: Set(now),
        updated_at: Set(now),
    }
}

async fn credit_wallet_and_insert_transaction(
    database: &Database,
    wallet: &types::wallet::Wallet,
    order: &RechargeOrderRecord,
    now: time::OffsetDateTime,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    update_wallet_in_tx(credited_wallet(wallet, order), tx).await?;
    insert_transaction_in_tx(payment_transaction(database.next_id(), wallet, order, now), tx).await
}

fn credited_wallet(wallet: &types::wallet::Wallet, order: &RechargeOrderRecord) -> types::wallet::Wallet {
    types::wallet::Wallet {
        recharge_balance: wallet.recharge_balance + order.recharge_amount,
        gift_balance: wallet.gift_balance + order.gift_amount,
        total_recharged: wallet.total_recharged + order.recharge_amount,
        total_adjusted: wallet.total_adjusted + order.gift_amount,
        ..wallet.clone()
    }
}

async fn update_wallet_in_tx(wallet: types::wallet::Wallet, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let record = wallet_records::Entity::find_by_id(wallet.id.0.clone())
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: WalletActiveModel = record.into();
    active.recharge_balance = Set(wallet.recharge_balance);
    active.gift_balance = Set(wallet.gift_balance);
    active.total_recharged = Set(wallet.total_recharged);
    active.total_adjusted = Set(wallet.total_adjusted);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await?;
    Ok(())
}

fn payment_transaction(
    id: String,
    wallet: &types::wallet::Wallet,
    order: &RechargeOrderRecord,
    now: time::OffsetDateTime,
) -> wallet_transaction_records::ActiveModel {
    let after_recharge = wallet.recharge_balance + order.recharge_amount;
    let after_gift = wallet.gift_balance + order.gift_amount;
    wallet_transaction_records::ActiveModel {
        id: Set(id),
        wallet_id: Set(wallet.id.0.clone()),
        category: Set(RECHARGE_CATEGORY.into()),
        reason_code: Set(TOPUP_GATEWAY_REASON.into()),
        amount: Set(order.total_arrival_amount),
        balance_before: Set(wallet.recharge_balance + wallet.gift_balance),
        balance_after: Set(after_recharge + after_gift),
        recharge_balance_before: Set(wallet.recharge_balance),
        recharge_balance_after: Set(after_recharge),
        gift_balance_before: Set(wallet.gift_balance),
        gift_balance_after: Set(after_gift),
        link_type: Set(Some(PAYMENT_LINK_TYPE.into())),
        link_id: Set(Some(order.id.clone())),
        operator_id: Set(Some(order.user_id.clone())),
        description: Set(Some(format!("Payment order {}", order.order_no))),
        created_at: Set(now),
    }
}

async fn insert_transaction_in_tx(transaction: wallet_transaction_records::ActiveModel, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    transaction.insert(tx).await?;
    Ok(())
}

async fn settle_affiliate_commission(
    database: &Database,
    order: &RechargeOrderRecord,
    now: time::OffsetDateTime,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    let Some(context) = affiliate_commission_context(order, tx).await? else {
        return Ok(());
    };
    if context.commission_below_minimum() {
        return insert_affiliate_commission(
            AffiliateCommissionInsert {
                database,
                context: &context,
                outcome: AffiliateCommissionOutcome::Failed(AFFILIATE_COMMISSION_FAILURE_BELOW_MINIMUM),
                now,
            },
            tx,
        )
        .await;
    }
    let referrer_wallet = ensure_wallet_in_tx(database, &context.referrer_user_id, tx).await?;
    let transaction_id = database.next_id();
    credit_affiliate_wallet_and_insert_transaction(&referrer_wallet, &context, &transaction_id, now, tx).await?;
    insert_affiliate_commission(
        AffiliateCommissionInsert {
            database,
            context: &context,
            outcome: AffiliateCommissionOutcome::Success(&transaction_id),
            now,
        },
        tx,
    )
    .await
}

async fn affiliate_commission_context(order: &RechargeOrderRecord, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<AffiliateCommissionContext>> {
    let Some(settings) = affiliate_commission_settings(tx).await? else {
        return Ok(None);
    };
    if settings.percent <= Decimal::ZERO {
        return Ok(None);
    }
    let user = UserEntity::find_by_id(order.user_id.clone())
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let Some(referrer_user_id) = valid_referrer_user_id(&user, order) else {
        return Ok(None);
    };
    Ok(Some(AffiliateCommissionContext::new(order, referrer_user_id, settings)))
}

async fn affiliate_commission_settings(tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<AffiliateCommissionSettings>> {
    let record = system_setting_records::Entity::find_by_id(SYSTEM_SETTINGS_ID.to_owned())
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    Ok(active_affiliate_commission_settings(
        record.affiliate_enabled,
        record.affiliate_commission_percent,
        record.affiliate_min_commission_amount,
    ))
}

fn active_affiliate_commission_settings(
    affiliate_enabled: bool,
    commission_percent: Decimal,
    min_commission_amount: Decimal,
) -> Option<AffiliateCommissionSettings> {
    affiliate_enabled.then_some(AffiliateCommissionSettings {
        percent: commission_percent,
        min_amount: min_commission_amount,
    })
}

fn valid_referrer_user_id(user: &UserRecord, order: &RechargeOrderRecord) -> Option<String> {
    user.referred_by_user_id.as_ref().filter(|referrer_id| *referrer_id != &order.user_id).cloned()
}

async fn credit_affiliate_wallet_and_insert_transaction(
    wallet: &types::wallet::Wallet,
    context: &AffiliateCommissionContext,
    transaction_id: &str,
    now: time::OffsetDateTime,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<()> {
    update_wallet_in_tx(affiliate_credited_wallet(wallet, context.commission_amount), tx).await?;
    insert_transaction_in_tx(affiliate_transaction(transaction_id.to_owned(), wallet, context, now), tx).await
}

fn affiliate_credited_wallet(wallet: &types::wallet::Wallet, amount: Decimal) -> types::wallet::Wallet {
    types::wallet::Wallet {
        gift_balance: wallet.gift_balance + amount,
        total_adjusted: wallet.total_adjusted + amount,
        ..wallet.clone()
    }
}

fn affiliate_transaction(
    id: String,
    wallet: &types::wallet::Wallet,
    context: &AffiliateCommissionContext,
    now: time::OffsetDateTime,
) -> wallet_transaction_records::ActiveModel {
    let after_gift = wallet.gift_balance + context.commission_amount;
    wallet_transaction_records::ActiveModel {
        id: Set(id),
        wallet_id: Set(wallet.id.0.clone()),
        category: Set(GIFT_CATEGORY.into()),
        reason_code: Set(AFFILIATE_COMMISSION_REASON.into()),
        amount: Set(context.commission_amount),
        balance_before: Set(wallet.recharge_balance + wallet.gift_balance),
        balance_after: Set(wallet.recharge_balance + after_gift),
        recharge_balance_before: Set(wallet.recharge_balance),
        recharge_balance_after: Set(wallet.recharge_balance),
        gift_balance_before: Set(wallet.gift_balance),
        gift_balance_after: Set(after_gift),
        link_type: Set(Some(AFFILIATE_LINK_TYPE.into())),
        link_id: Set(Some(context.recharge_order_id.clone())),
        operator_id: Set(Some(context.referred_user_id.clone())),
        description: Set(Some(format!("Affiliate commission for order {}", context.order_no))),
        created_at: Set(now),
    }
}

async fn insert_affiliate_commission(input: AffiliateCommissionInsert<'_>, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    affiliate_commission_records::ActiveModel {
        id: Set(input.database.next_id()),
        referrer_user_id: Set(input.context.referrer_user_id.clone()),
        referred_user_id: Set(input.context.referred_user_id.clone()),
        recharge_order_id: Set(input.context.recharge_order_id.clone()),
        payable_amount: Set(input.context.payable_amount),
        commission_percent: Set(input.context.commission_percent),
        commission_amount: Set(input.context.commission_amount),
        wallet_transaction_id: Set(input.outcome.wallet_transaction_id()),
        status: Set(input.outcome.status().to_owned()),
        failure_reason: Set(input.outcome.failure_reason()),
        created_at: Set(input.now),
    }
    .insert(tx)
    .await?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AffiliateCommissionSettings {
    percent: Decimal,
    min_amount: Decimal,
}

struct AffiliateCommissionContext {
    referrer_user_id: String,
    referred_user_id: String,
    recharge_order_id: String,
    order_no: String,
    payable_amount: Decimal,
    commission_percent: Decimal,
    commission_amount: Decimal,
    min_commission_amount: Decimal,
}

impl AffiliateCommissionContext {
    fn new(order: &RechargeOrderRecord, referrer_user_id: String, settings: AffiliateCommissionSettings) -> Self {
        Self {
            referrer_user_id,
            referred_user_id: order.user_id.clone(),
            recharge_order_id: order.id.clone(),
            order_no: order.order_no.clone(),
            payable_amount: order.payable_amount,
            commission_percent: settings.percent,
            commission_amount: order.payable_amount * settings.percent / Decimal::new(PERCENT_DENOMINATOR, 0),
            min_commission_amount: settings.min_amount,
        }
    }

    fn commission_below_minimum(&self) -> bool {
        self.commission_amount < self.min_commission_amount
    }
}

struct AffiliateCommissionInsert<'a> {
    database: &'a Database,
    context: &'a AffiliateCommissionContext,
    outcome: AffiliateCommissionOutcome<'a>,
    now: time::OffsetDateTime,
}

enum AffiliateCommissionOutcome<'a> {
    Success(&'a str),
    Failed(&'a str),
}

impl AffiliateCommissionOutcome<'_> {
    fn wallet_transaction_id(&self) -> Option<String> {
        match self {
            Self::Success(transaction_id) => Some((*transaction_id).to_owned()),
            Self::Failed(_) => None,
        }
    }

    fn status(&self) -> &str {
        match self {
            Self::Success(_) => AFFILIATE_COMMISSION_STATUS_SUCCESS,
            Self::Failed(_) => AFFILIATE_COMMISSION_STATUS_FAILED,
        }
    }

    fn failure_reason(&self) -> Option<String> {
        match self {
            Self::Success(_) => None,
            Self::Failed(reason) => Some((*reason).to_owned()),
        }
    }
}

#[cfg(test)]
#[path = "settlement_tests.rs"]
mod tests;
