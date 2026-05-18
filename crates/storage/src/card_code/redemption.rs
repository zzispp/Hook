use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait, sea_query::Expr};
use types::{
    card_code::{CARD_CODE_STATUS_ACTIVE, CARD_CODE_STATUS_EXPIRED, CARD_CODE_STATUS_USED, CardCodeRedeemInput, CardCodeRedeemRecord},
    wallet::Wallet,
};

use crate::{
    Database, StorageError, StorageResult,
    card_code::{
        CardCodeRecord, card_code_records, query,
        redemption_currency::{RedemptionAmounts, accounting_redemption_amounts, wallet_in_accounting_currency},
    },
    wallet::{wallet_records, wallet_records::ActiveModel as WalletActiveModel, wallet_transaction_records},
};

const DEFAULT_WALLET_STATUS: &str = "active";
const DEFAULT_WALLET_LIMIT_MODE: &str = "finite";
const CARD_CODE_LINK_TYPE: &str = "card_code";
const TOPUP_CARD_CODE_REASON: &str = "topup_card_code";
const GIFT_CARD_CODE_REASON: &str = "gift_card_code";

pub(super) async fn redeem(database: &Database, input: CardCodeRedeemInput) -> StorageResult<CardCodeRedeemRecord> {
    ensure_accounting_currency(&input.target_currency)?;
    let tx = database.connection().begin().await?;
    let now = time::OffsetDateTime::now_utc();
    mark_code_used(&input, now, &tx).await?;
    let code = find_code_by_code_in_tx(&input.code, &tx).await?.ok_or(StorageError::NotFound)?;
    let wallet = ensure_wallet_in_tx(database, &input.user_id, &input.target_currency, &tx).await?;
    let wallet = wallet_in_accounting_currency(wallet)?;
    let amounts = accounting_redemption_amounts(&code)?;
    let transaction = build_transaction(database.next_id(), &wallet, &code, &input, amounts, now);
    let updated_wallet = credited_wallet(wallet, amounts, &input.target_currency);
    update_wallet_in_tx(updated_wallet, &tx).await?;
    let transaction = insert_transaction_in_tx(transaction, &tx).await?;
    let code = attach_redeem_wallet(code, &transaction, &tx).await?;
    tx.commit().await?;
    Ok(CardCodeRedeemRecord {
        card_code: code.into(),
        transaction: transaction.into(),
    })
}

fn ensure_accounting_currency(currency: &str) -> StorageResult<()> {
    if currency == currency::ACCOUNTING_CURRENCY {
        return Ok(());
    }
    Err(StorageError::Conflict(format!(
        "card code redemption target currency must be {}",
        currency::ACCOUNTING_CURRENCY
    )))
}

async fn mark_code_used(input: &CardCodeRedeemInput, now: time::OffsetDateTime, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let result = card_code_records::Entity::update_many()
        .col_expr(card_code_records::Column::Status, Expr::value(CARD_CODE_STATUS_USED))
        .col_expr(card_code_records::Column::UsedByUserId, Expr::value(input.user_id.clone()))
        .col_expr(card_code_records::Column::UsedByUsername, Expr::value(input.username.clone()))
        .col_expr(card_code_records::Column::UsedIp, Expr::value(input.client_ip.clone()))
        .col_expr(card_code_records::Column::UsedAt, Expr::value(now))
        .col_expr(card_code_records::Column::UpdatedAt, Expr::value(now))
        .filter(card_code_records::Column::Code.eq(input.code.as_str()))
        .filter(card_code_records::Column::Status.eq(CARD_CODE_STATUS_ACTIVE))
        .filter(card_code_records::Column::UsedAt.is_null())
        .filter(query::not_expired_condition(now))
        .exec(tx)
        .await?;
    if result.rows_affected == 1 {
        return Ok(());
    }
    reject_unusable_code(input.code.as_str(), now, tx).await
}

async fn reject_unusable_code(code: &str, now: time::OffsetDateTime, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let Some(record) = find_code_by_code_in_tx(code, tx).await? else {
        return Err(StorageError::NotFound);
    };
    if record.status == CARD_CODE_STATUS_ACTIVE && record.used_at.is_none() && is_expired(&record, now) {
        expire_code(record, now, tx).await?;
        return Err(StorageError::Conflict("card code expired".into()));
    }
    Err(StorageError::Conflict(format!("card code is {}", record.status)))
}

async fn expire_code(record: CardCodeRecord, now: time::OffsetDateTime, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let mut active: card_code_records::ActiveModel = record.into();
    active.status = Set(CARD_CODE_STATUS_EXPIRED.into());
    active.updated_at = Set(now);
    active.update(tx).await?;
    Ok(())
}

fn credited_wallet(wallet: Wallet, amounts: RedemptionAmounts, target_currency: &str) -> Wallet {
    Wallet {
        recharge_balance: wallet.recharge_balance + amounts.recharge,
        gift_balance: wallet.gift_balance + amounts.gift,
        currency: target_currency.into(),
        total_recharged: wallet.total_recharged + amounts.recharge,
        total_adjusted: wallet.total_adjusted + amounts.gift,
        ..wallet
    }
}

fn build_transaction(
    id: String,
    wallet: &Wallet,
    code: &CardCodeRecord,
    input: &CardCodeRedeemInput,
    amounts: RedemptionAmounts,
    now: time::OffsetDateTime,
) -> wallet_transaction_records::ActiveModel {
    let after_recharge = wallet.recharge_balance + amounts.recharge;
    let after_gift = wallet.gift_balance + amounts.gift;
    wallet_transaction_records::ActiveModel {
        id: Set(id),
        wallet_id: Set(wallet.id.0.clone()),
        category: Set(transaction_category(code)),
        reason_code: Set(transaction_reason(code)),
        amount: Set(amounts.total()),
        balance_before: Set(wallet.recharge_balance + wallet.gift_balance),
        balance_after: Set(after_recharge + after_gift),
        recharge_balance_before: Set(wallet.recharge_balance),
        recharge_balance_after: Set(after_recharge),
        gift_balance_before: Set(wallet.gift_balance),
        gift_balance_after: Set(after_gift),
        link_type: Set(Some(CARD_CODE_LINK_TYPE.into())),
        link_id: Set(Some(code.id.clone())),
        operator_id: Set(Some(input.user_id.clone())),
        description: Set(Some(format!("Card code {} redeemed", mask_code(&code.code)))),
        created_at: Set(now),
    }
}

fn transaction_category(code: &CardCodeRecord) -> String {
    if code.recharge_amount > Decimal::ZERO {
        return "recharge".into();
    }
    "gift".into()
}

fn transaction_reason(code: &CardCodeRecord) -> String {
    if code.recharge_amount > Decimal::ZERO {
        return TOPUP_CARD_CODE_REASON.into();
    }
    GIFT_CARD_CODE_REASON.into()
}

async fn find_code_by_code_in_tx(code: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<CardCodeRecord>> {
    card_code_records::Entity::find()
        .filter(card_code_records::Column::Code.eq(code))
        .one(tx)
        .await
        .map_err(StorageError::from)
}

async fn attach_redeem_wallet(
    record: CardCodeRecord,
    transaction: &wallet_transaction_records::Model,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<CardCodeRecord> {
    let mut active: card_code_records::ActiveModel = record.into();
    active.wallet_id = Set(Some(transaction.wallet_id.clone()));
    active.wallet_transaction_id = Set(Some(transaction.id.clone()));
    Ok(active.update(tx).await?)
}

async fn ensure_wallet_in_tx(database: &Database, user_id: &str, currency: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Wallet> {
    if let Some(wallet) = wallet_by_user_in_tx(user_id, tx).await? {
        return Ok(wallet);
    }
    insert_wallet_in_tx(database, user_id, currency, tx).await?;
    wallet_by_user_in_tx(user_id, tx).await?.ok_or(StorageError::NotFound)
}

async fn wallet_by_user_in_tx(user_id: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<Option<Wallet>> {
    wallet_records::Entity::find()
        .filter(wallet_records::Column::UserId.eq(user_id))
        .one(tx)
        .await
        .map(|record| record.map(Wallet::from))
        .map_err(StorageError::from)
}

async fn insert_wallet_in_tx(database: &Database, user_id: &str, currency: &str, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let now = time::OffsetDateTime::now_utc();
    wallet_records::Entity::insert(wallet_active_model(database.next_id(), user_id, currency, now))
        .on_conflict_do_nothing_on([wallet_records::Column::UserId])
        .exec_without_returning(tx)
        .await?;
    Ok(())
}

fn wallet_active_model(id: String, user_id: &str, currency: &str, now: time::OffsetDateTime) -> WalletActiveModel {
    WalletActiveModel {
        id: Set(id),
        user_id: Set(user_id.to_owned()),
        recharge_balance: Set(Decimal::ZERO),
        gift_balance: Set(Decimal::ZERO),
        currency: Set(currency.into()),
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

async fn update_wallet_in_tx(wallet: Wallet, tx: &sea_orm::DatabaseTransaction) -> StorageResult<()> {
    let record = wallet_records::Entity::find_by_id(wallet.id.0.clone())
        .one(tx)
        .await?
        .ok_or(StorageError::NotFound)?;
    let mut active: WalletActiveModel = record.into();
    active.recharge_balance = Set(wallet.recharge_balance);
    active.gift_balance = Set(wallet.gift_balance);
    active.currency = Set(wallet.currency);
    active.total_recharged = Set(wallet.total_recharged);
    active.total_consumed = Set(wallet.total_consumed);
    active.total_refunded = Set(wallet.total_refunded);
    active.total_adjusted = Set(wallet.total_adjusted);
    active.updated_at = Set(time::OffsetDateTime::now_utc());
    active.update(tx).await?;
    Ok(())
}

async fn insert_transaction_in_tx(
    transaction: wallet_transaction_records::ActiveModel,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<wallet_transaction_records::Model> {
    transaction.insert(tx).await.map_err(StorageError::from)
}

fn is_expired(record: &CardCodeRecord, now: time::OffsetDateTime) -> bool {
    record.expires_at.is_some_and(|expires_at| expires_at <= now)
}

fn mask_code(code: &str) -> String {
    if code.len() <= 8 {
        return code.into();
    }
    format!("{}...{}", &code[..4], &code[code.len() - 4..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_accounting_target_currency() {
        let error = ensure_accounting_currency("CNY").unwrap_err();

        assert!(matches!(error, StorageError::Conflict(message) if message.contains("target currency must be USD")));
    }
}
