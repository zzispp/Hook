use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

const DEFAULT_CURRENCY: &str = "CNY";
const DEFAULT_WALLET_STATUS: &str = "active";
const DEFAULT_WALLET_LIMIT_MODE: &str = "finite";
const DECIMAL_PRECISION: u32 = 20;
const DECIMAL_SCALE: u32 = 8;

pub(super) fn wallets_table() -> TableCreateStatement {
    let mut table = Table::create();
    let mut user_fk = user_fk(Wallets::Table, Wallets::UserId, ForeignKeyAction::Cascade);
    table
        .table(Wallets::Table)
        .if_not_exists()
        .col(string_len(Wallets::Id, 36).primary_key())
        .col(string_len(Wallets::UserId, 36))
        .col(wallet_decimal(Wallets::RechargeBalance).default(0))
        .col(wallet_decimal(Wallets::GiftBalance).default(0))
        .col(string_len(Wallets::Currency, 3).default(DEFAULT_CURRENCY))
        .col(string_len(Wallets::Status, 20).default(DEFAULT_WALLET_STATUS))
        .col(string_len(Wallets::LimitMode, 20).default(DEFAULT_WALLET_LIMIT_MODE))
        .col(wallet_decimal(Wallets::TotalRecharged).default(0))
        .col(wallet_decimal(Wallets::TotalConsumed).default(0))
        .col(wallet_decimal(Wallets::TotalRefunded).default(0))
        .col(wallet_decimal(Wallets::TotalAdjusted).default(0))
        .col(timestamp_tz(Wallets::CreatedAt))
        .col(timestamp_tz(Wallets::UpdatedAt))
        .foreign_key(&mut user_fk);
    for check in wallet_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

pub(super) fn wallet_transactions_table() -> TableCreateStatement {
    let mut table = Table::create();
    let mut wallet_fk = wallet_fk();
    table
        .table(WalletTransactions::Table)
        .if_not_exists()
        .col(string_len(WalletTransactions::Id, 36).primary_key())
        .col(string_len(WalletTransactions::WalletId, 36))
        .col(string_len(WalletTransactions::Category, 20))
        .col(string_len(WalletTransactions::ReasonCode, 40))
        .col(wallet_decimal(WalletTransactions::Amount))
        .col(wallet_decimal(WalletTransactions::BalanceBefore))
        .col(wallet_decimal(WalletTransactions::BalanceAfter))
        .col(wallet_decimal(WalletTransactions::RechargeBalanceBefore))
        .col(wallet_decimal(WalletTransactions::RechargeBalanceAfter))
        .col(wallet_decimal(WalletTransactions::GiftBalanceBefore))
        .col(wallet_decimal(WalletTransactions::GiftBalanceAfter))
        .col(string_len_null(WalletTransactions::LinkType, 30))
        .col(string_len_null(WalletTransactions::LinkId, 100))
        .col(string_len_null(WalletTransactions::OperatorId, 36))
        .col(text_null(WalletTransactions::Description))
        .col(timestamp_tz(WalletTransactions::CreatedAt))
        .foreign_key(&mut wallet_fk);
    for check in wallet_transaction_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

fn user_fk<T, C>(table: T, column: C, on_delete: ForeignKeyAction) -> ForeignKeyCreateStatement
where
    T: Iden + 'static,
    C: Iden + 'static,
{
    let mut foreign_key = ForeignKey::create();
    foreign_key.from(table, column).to(Users::Table, Users::Id).on_delete(on_delete);
    foreign_key
}

fn wallet_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(WalletTransactions::Table, WalletTransactions::WalletId)
        .to(Wallets::Table, Wallets::Id)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn wallet_decimal<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    decimal_len(col, DECIMAL_PRECISION, DECIMAL_SCALE)
}

fn wallet_checks() -> [&'static str; 8] {
    [
        r#""currency" = 'CNY'"#,
        r#""status" IN ('active', 'disabled')"#,
        r#""limit_mode" IN ('finite', 'unlimited')"#,
        r#""recharge_balance" >= 0"#,
        r#""gift_balance" >= 0"#,
        r#""total_recharged" >= 0"#,
        r#""total_consumed" >= 0"#,
        r#""total_refunded" >= 0"#,
    ]
}

fn wallet_transaction_checks() -> [&'static str; 7] {
    [
        r#""category" IN ('recharge', 'gift', 'adjust', 'refund', 'consume')"#,
        r#""balance_before" = "recharge_balance_before" + "gift_balance_before""#,
        r#""balance_after" = "recharge_balance_after" + "gift_balance_after""#,
        r#""recharge_balance_before" >= 0"#,
        r#""recharge_balance_after" >= 0"#,
        r#""gift_balance_before" >= 0"#,
        r#""gift_balance_after" >= 0"#,
    ]
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
