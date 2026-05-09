use sea_orm_migration::{prelude::*, schema::*};

mod rbac_seed;

const DEFAULT_CURRENCY: &str = "CNY";
const DEFAULT_STATUS: &str = "active";
const DEFAULT_LIMIT_MODE: &str = "finite";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        create_wallets(manager).await?;
        create_wallet_transactions(manager).await?;
        create_indices(manager).await?;
        backfill_user_wallets(manager).await?;
        rbac_seed::seed_wallet_rbac(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        rbac_seed::delete_wallet_rbac(manager).await?;
        manager
            .drop_table(Table::drop().table(WalletTransactions::Table).if_exists().to_owned())
            .await?;
        manager.drop_table(Table::drop().table(Wallets::Table).if_exists().to_owned()).await
    }
}

async fn create_wallets(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(Wallets::Table)
                .if_not_exists()
                .col(string_len(Wallets::Id, 36).primary_key())
                .col(string_len(Wallets::UserId, 36))
                .col(decimal_len(Wallets::RechargeBalance, 20, 8).default(0))
                .col(decimal_len(Wallets::GiftBalance, 20, 8).default(0))
                .col(string_len(Wallets::Currency, 3).default(DEFAULT_CURRENCY))
                .col(string_len(Wallets::Status, 20).default(DEFAULT_STATUS))
                .col(string_len(Wallets::LimitMode, 20).default(DEFAULT_LIMIT_MODE))
                .col(decimal_len(Wallets::TotalRecharged, 20, 8).default(0))
                .col(decimal_len(Wallets::TotalConsumed, 20, 8).default(0))
                .col(decimal_len(Wallets::TotalRefunded, 20, 8).default(0))
                .col(decimal_len(Wallets::TotalAdjusted, 20, 8).default(0))
                .col(timestamp_tz(Wallets::CreatedAt))
                .col(timestamp_tz(Wallets::UpdatedAt))
                .foreign_key(
                    ForeignKey::create()
                        .from(Wallets::Table, Wallets::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .check(Expr::cust(r#""currency" = 'CNY'"#))
                .check(Expr::cust(r#""status" IN ('active', 'disabled')"#))
                .check(Expr::cust(r#""limit_mode" IN ('finite', 'unlimited')"#))
                .check(Expr::cust(r#""recharge_balance" >= 0"#))
                .check(Expr::cust(r#""gift_balance" >= 0"#))
                .check(Expr::cust(r#""total_recharged" >= 0"#))
                .check(Expr::cust(r#""total_consumed" >= 0"#))
                .check(Expr::cust(r#""total_refunded" >= 0"#))
                .to_owned(),
        )
        .await
}

async fn create_wallet_transactions(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .create_table(
            Table::create()
                .table(WalletTransactions::Table)
                .if_not_exists()
                .col(string_len(WalletTransactions::Id, 36).primary_key())
                .col(string_len(WalletTransactions::WalletId, 36))
                .col(string_len(WalletTransactions::Category, 20))
                .col(string_len(WalletTransactions::ReasonCode, 40))
                .col(decimal_len(WalletTransactions::Amount, 20, 8))
                .col(decimal_len(WalletTransactions::BalanceBefore, 20, 8))
                .col(decimal_len(WalletTransactions::BalanceAfter, 20, 8))
                .col(decimal_len(WalletTransactions::RechargeBalanceBefore, 20, 8))
                .col(decimal_len(WalletTransactions::RechargeBalanceAfter, 20, 8))
                .col(decimal_len(WalletTransactions::GiftBalanceBefore, 20, 8))
                .col(decimal_len(WalletTransactions::GiftBalanceAfter, 20, 8))
                .col(string_len_null(WalletTransactions::LinkType, 30))
                .col(string_len_null(WalletTransactions::LinkId, 100))
                .col(string_len_null(WalletTransactions::OperatorId, 36))
                .col(text_null(WalletTransactions::Description))
                .col(timestamp_tz(WalletTransactions::CreatedAt))
                .foreign_key(
                    ForeignKey::create()
                        .from(WalletTransactions::Table, WalletTransactions::WalletId)
                        .to(Wallets::Table, Wallets::Id)
                        .on_delete(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .from(WalletTransactions::Table, WalletTransactions::OperatorId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::SetNull),
                )
                .check(Expr::cust(r#""category" IN ('recharge', 'gift', 'adjust', 'refund', 'consume')"#))
                .check(Expr::cust(r#""balance_before" = "recharge_balance_before" + "gift_balance_before""#))
                .check(Expr::cust(r#""balance_after" = "recharge_balance_after" + "gift_balance_after""#))
                .check(Expr::cust(r#""recharge_balance_before" >= 0"#))
                .check(Expr::cust(r#""recharge_balance_after" >= 0"#))
                .check(Expr::cust(r#""gift_balance_before" >= 0"#))
                .check(Expr::cust(r#""gift_balance_after" >= 0"#))
                .to_owned(),
        )
        .await
}

async fn create_indices(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    for index in wallet_indices() {
        manager.create_index(index).await?;
    }
    Ok(())
}

fn wallet_indices() -> Vec<IndexCreateStatement> {
    vec![
        unique_index("index_wallets_by_user_id", Wallets::Table, Wallets::UserId),
        index("index_wallets_by_status", Wallets::Table, Wallets::Status),
        compound_index(
            "index_wallet_transactions_by_wallet_created",
            WalletTransactions::Table,
            WalletTransactions::WalletId,
            WalletTransactions::CreatedAt,
        ),
        compound_index(
            "index_wallet_transactions_by_link",
            WalletTransactions::Table,
            WalletTransactions::LinkType,
            WalletTransactions::LinkId,
        ),
        compound_index(
            "index_wallet_transactions_by_category_created",
            WalletTransactions::Table,
            WalletTransactions::Category,
            WalletTransactions::CreatedAt,
        ),
    ]
}

async fn backfill_user_wallets(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let sql = r#"
        INSERT INTO wallets (
            id, user_id, recharge_balance, gift_balance, currency, status, limit_mode,
            total_recharged, total_consumed, total_refunded, total_adjusted, created_at, updated_at
        )
        SELECT
            format(
                '%s-%s-%s-%s-%s',
                substr(md5(users.id || ':wallet'), 1, 8),
                substr(md5(users.id || ':wallet'), 9, 4),
                substr(md5(users.id || ':wallet'), 13, 4),
                substr(md5(users.id || ':wallet'), 17, 4),
                substr(md5(users.id || ':wallet'), 21, 12)
            ),
            users.id,
            0,
            0,
            'CNY',
            'active',
            'finite',
            0,
            0,
            0,
            0,
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP
        FROM users
        LEFT JOIN wallets ON wallets.user_id = users.id
        WHERE wallets.id IS NULL
    "#;
    manager.get_connection().execute_unprepared(sql).await.map(|_| ())
}

fn index<T, C>(name: &str, table: T, column: C) -> IndexCreateStatement
where
    T: Iden + 'static,
    C: Iden + 'static,
{
    Index::create().name(name).table(table).col(column).if_not_exists().to_owned()
}

fn unique_index<T, C>(name: &str, table: T, column: C) -> IndexCreateStatement
where
    T: Iden + 'static,
    C: Iden + 'static,
{
    Index::create().name(name).table(table).col(column).unique().if_not_exists().to_owned()
}

fn compound_index<T, C1, C2>(name: &str, table: T, first: C1, second: C2) -> IndexCreateStatement
where
    T: Iden + 'static,
    C1: Iden + 'static,
    C2: Iden + 'static,
{
    Index::create().name(name).table(table).col(first).col(second).if_not_exists().to_owned()
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Wallets {
    Table,
    Id,
    UserId,
    RechargeBalance,
    GiftBalance,
    Currency,
    Status,
    LimitMode,
    TotalRecharged,
    TotalConsumed,
    TotalRefunded,
    TotalAdjusted,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WalletTransactions {
    Table,
    Id,
    WalletId,
    Category,
    ReasonCode,
    Amount,
    BalanceBefore,
    BalanceAfter,
    RechargeBalanceBefore,
    RechargeBalanceAfter,
    GiftBalanceBefore,
    GiftBalanceAfter,
    LinkType,
    LinkId,
    OperatorId,
    Description,
    CreatedAt,
}
