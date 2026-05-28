use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

const DECIMAL_PRECISION: u32 = 20;
const DECIMAL_SCALE: u32 = 8;

pub(super) fn recharge_tables() -> Vec<TableCreateStatement> {
    vec![
        recharge_packages_table(),
        recharge_orders_table(),
        payment_channels_table(),
        payment_callback_records_table(),
    ]
}

fn recharge_packages_table() -> TableCreateStatement {
    let mut table = Table::create();
    table
        .table(RechargePackages::Table)
        .if_not_exists()
        .col(string_len(RechargePackages::Id, 36).primary_key())
        .col(string_len(RechargePackages::Name, 100))
        .col(text_null(RechargePackages::Description))
        .col(recharge_decimal(RechargePackages::RechargeAmount))
        .col(recharge_decimal(RechargePackages::GiftAmount).default(0))
        .col(string_len(RechargePackages::Status, 20).default("active"))
        .col(big_integer(RechargePackages::SortOrder).default(0))
        .col(timestamp_tz(RechargePackages::CreatedAt))
        .col(timestamp_tz(RechargePackages::UpdatedAt));
    for check in package_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

fn recharge_orders_table() -> TableCreateStatement {
    let mut table = Table::create();
    let mut user_fk = user_fk();
    let mut package_fk = package_fk();
    table
        .table(RechargeOrders::Table)
        .if_not_exists()
        .col(string_len(RechargeOrders::Id, 36).primary_key())
        .col(string_len(RechargeOrders::OrderNo, 64))
        .col(string_len(RechargeOrders::UserId, 36))
        .col(string_len_null(RechargeOrders::PackageId, 36))
        .col(string_len(RechargeOrders::PackageName, 100))
        .col(recharge_decimal(RechargeOrders::RechargeAmount))
        .col(recharge_decimal(RechargeOrders::GiftAmount).default(0))
        .col(recharge_decimal(RechargeOrders::TotalArrivalAmount))
        .col(recharge_decimal(RechargeOrders::PayableAmount))
        .col(string_len(RechargeOrders::Status, 20).default("pending"))
        .col(string_len_null(RechargeOrders::PaymentChannelCode, 64))
        .col(string_len_null(RechargeOrders::PaymentChannelName, 100))
        .col(string_len_null(RechargeOrders::PaymentMethod, 50))
        .col(string_len_null(RechargeOrders::ProviderTradeNo, 128))
        .col(text_null(RechargeOrders::PaymentRequestJson))
        .col(string_len_null(RechargeOrders::RefundStatus, 20))
        .col(recharge_decimal(RechargeOrders::RefundAmount).null())
        .col(timestamp_tz_null(RechargeOrders::PaidAt))
        .col(timestamp_tz_null(RechargeOrders::RefundedAt))
        .col(timestamp_tz(RechargeOrders::ExpiresAt))
        .col(timestamp_tz(RechargeOrders::CreatedAt))
        .col(timestamp_tz(RechargeOrders::UpdatedAt))
        .foreign_key(&mut user_fk)
        .foreign_key(&mut package_fk);
    for check in order_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

fn payment_channels_table() -> TableCreateStatement {
    Table::create()
        .table(PaymentChannels::Table)
        .if_not_exists()
        .col(string_len(PaymentChannels::Code, 64).primary_key())
        .col(string_len(PaymentChannels::Name, 100))
        .col(boolean(PaymentChannels::Enabled).default(false))
        .col(text(PaymentChannels::ConfigJson).default("{}"))
        .col(text(PaymentChannels::EncryptedSecret).default(""))
        .col(timestamp_tz(PaymentChannels::RegisteredAt))
        .col(timestamp_tz(PaymentChannels::UpdatedAt))
        .to_owned()
}

fn payment_callback_records_table() -> TableCreateStatement {
    let mut table = Table::create();
    table
        .table(PaymentCallbackRecords::Table)
        .if_not_exists()
        .col(string_len(PaymentCallbackRecords::Id, 36).primary_key())
        .col(string_len(PaymentCallbackRecords::PaymentChannelCode, 64))
        .col(string_len(PaymentCallbackRecords::CallbackKind, 20))
        .col(string_len(PaymentCallbackRecords::HttpMethod, 10))
        .col(string_len_null(PaymentCallbackRecords::OrderNo, 64))
        .col(string_len_null(PaymentCallbackRecords::ProviderTradeNo, 128))
        .col(string_len_null(PaymentCallbackRecords::PaymentMethod, 50))
        .col(string_len_null(PaymentCallbackRecords::TradeStatus, 50))
        .col(string_len(PaymentCallbackRecords::Status, 20).default("received"))
        .col(boolean(PaymentCallbackRecords::Settled).default(false))
        .col(text_null(PaymentCallbackRecords::ErrorMessage))
        .col(text(PaymentCallbackRecords::RawParamsJson).default("{}"))
        .col(timestamp_tz(PaymentCallbackRecords::ReceivedAt))
        .col(timestamp_tz_null(PaymentCallbackRecords::ProcessedAt));
    for check in payment_callback_checks() {
        table.check(Expr::cust(check));
    }
    table.to_owned()
}

fn payment_callback_checks() -> [&'static str; 2] {
    [
        r#""callback_kind" IN ('notify', 'return')"#,
        r#""status" IN ('received', 'processed', 'ignored', 'failed')"#,
    ]
}

fn user_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(RechargeOrders::Table, RechargeOrders::UserId)
        .to(Users::Table, Users::Id)
        .on_delete(ForeignKeyAction::Restrict);
    foreign_key
}

fn package_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .from(RechargeOrders::Table, RechargeOrders::PackageId)
        .to(RechargePackages::Table, RechargePackages::Id)
        .on_delete(ForeignKeyAction::SetNull);
    foreign_key
}

fn recharge_decimal<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    decimal_len(col, DECIMAL_PRECISION, DECIMAL_SCALE)
}

fn package_checks() -> [&'static str; 3] {
    [r#""status" IN ('active', 'disabled')"#, r#""recharge_amount" > 0"#, r#""gift_amount" >= 0"#]
}

fn order_checks() -> [&'static str; 5] {
    [
        r#""status" IN ('pending', 'expired', 'paid', 'cancelled', 'failed')"#,
        r#""recharge_amount" > 0"#,
        r#""gift_amount" >= 0"#,
        r#""total_arrival_amount" = "recharge_amount" + "gift_amount""#,
        r#""payable_amount" > 0"#,
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
