use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

const DECIMAL_PRECISION: u32 = 20;
const DECIMAL_SCALE: u32 = 8;

pub(super) fn card_code_types_table() -> TableCreateStatement {
    let mut table = Table::create();
    table
        .table(CardCodeTypes::Table)
        .if_not_exists()
        .col(string_len(CardCodeTypes::Id, 36).primary_key())
        .col(string_len(CardCodeTypes::Name, 100))
        .col(string_len(CardCodeTypes::BalanceType, 20))
        .col(string_len(CardCodeTypes::Status, 20).default("active"))
        .col(text_null(CardCodeTypes::Remark))
        .col(timestamp_tz(CardCodeTypes::CreatedAt))
        .col(timestamp_tz(CardCodeTypes::UpdatedAt));
    for check in card_code_type_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

pub(super) fn card_codes_table() -> TableCreateStatement {
    let mut table = Table::create();
    let mut type_fk = card_code_type_fk();
    let mut created_user_fk = user_fk(CardCodes::CreatedByUserId);
    let mut used_user_fk = user_fk(CardCodes::UsedByUserId);
    let mut wallet_fk = wallet_fk();
    let mut transaction_fk = wallet_transaction_fk();
    table
        .table(CardCodes::Table)
        .if_not_exists()
        .col(string_len(CardCodes::Id, 36).primary_key())
        .col(string_len(CardCodes::Code, 128))
        .col(string_len(CardCodes::BatchNo, 64))
        .col(string_len(CardCodes::TypeId, 36))
        .col(string_len(CardCodes::TypeName, 100))
        .col(card_decimal(CardCodes::RechargeAmount).default(0))
        .col(card_decimal(CardCodes::GiftAmount).default(0))
        .col(string_len(CardCodes::Status, 20).default("active"))
        .col(text_null(CardCodes::Remark))
        .col(timestamp_tz_null(CardCodes::ExpiresAt))
        .col(string_len_null(CardCodes::CreatedByUserId, 36))
        .col(string_len_null(CardCodes::CreatedByUsername, 100))
        .col(string_len_null(CardCodes::CreatedIp, 45))
        .col(string_len_null(CardCodes::UsedByUserId, 36))
        .col(string_len_null(CardCodes::UsedByUsername, 100))
        .col(string_len_null(CardCodes::UsedIp, 45))
        .col(timestamp_tz_null(CardCodes::UsedAt))
        .col(string_len_null(CardCodes::WalletId, 36))
        .col(string_len_null(CardCodes::WalletTransactionId, 36))
        .col(timestamp_tz(CardCodes::CreatedAt))
        .col(timestamp_tz(CardCodes::UpdatedAt))
        .foreign_key(&mut type_fk)
        .foreign_key(&mut created_user_fk)
        .foreign_key(&mut used_user_fk)
        .foreign_key(&mut wallet_fk)
        .foreign_key(&mut transaction_fk);
    for check in card_code_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

fn card_code_type_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(CardCodes::Table, CardCodes::TypeId)
        .to(CardCodeTypes::Table, CardCodeTypes::Id)
        .on_delete(ForeignKeyAction::Restrict);
    foreign_key
}

fn user_fk<C>(column: C) -> ForeignKeyCreateStatement
where
    C: Iden + 'static,
{
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(CardCodes::Table, column)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::SetNull);
    foreign_key
}

fn wallet_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(CardCodes::Table, CardCodes::WalletId)
        .to(Wallets::Table, Wallets::Id)
        .on_delete(ForeignKeyAction::SetNull);
    foreign_key
}

fn wallet_transaction_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(CardCodes::Table, CardCodes::WalletTransactionId)
        .to(WalletTransactions::Table, WalletTransactions::Id)
        .on_delete(ForeignKeyAction::SetNull);
    foreign_key
}

fn card_decimal<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    decimal_len(col, DECIMAL_PRECISION, DECIMAL_SCALE)
}

fn card_code_type_checks() -> [&'static str; 2] {
    [
        r#""status" IN ('active', 'disabled')"#,
        r#""balance_type" IN ('recharge', 'gift')"#,
    ]
}

fn card_code_checks() -> [&'static str; 4] {
    [
        r#""status" IN ('active', 'disabled', 'used', 'expired')"#,
        r#""recharge_amount" >= 0"#,
        r#""gift_amount" >= 0"#,
        r#""recharge_amount" + "gift_amount" > 0"#,
    ]
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}

fn timestamp_tz_null<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().null().take()
}
