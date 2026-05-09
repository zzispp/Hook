use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

const ADMIN_ROLE: &str = "admin";
const WALLET_CENTER_MENU_ID: &str = "00000000-0000-7000-8000-000000000208";
const WALLET_BALANCE_API_ID: &str = "00000000-0000-7000-8000-000000000337";
const WALLET_TRANSACTIONS_API_ID: &str = "00000000-0000-7000-8000-000000000338";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        delete_admin_wallet_bindings(manager).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

async fn delete_admin_wallet_bindings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        DELETE FROM role_api_permissions
        WHERE role_code = '{ADMIN_ROLE}'
          AND api_permission_id IN ('{WALLET_BALANCE_API_ID}', '{WALLET_TRANSACTIONS_API_ID}');

        DELETE FROM role_menu_permissions
        WHERE role_code = '{ADMIN_ROLE}'
          AND menu_item_id = '{WALLET_CENTER_MENU_ID}';
        "#
    );
    manager.get_connection().execute_unprepared(&sql).await.map(|_| ())
}
