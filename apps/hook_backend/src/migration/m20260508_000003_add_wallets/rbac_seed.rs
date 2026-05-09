use sea_orm::{ConnectionTrait, DatabaseTransaction, TransactionTrait};
use sea_orm_migration::prelude::*;

const ACCOUNT_SECTION_ID: &str = "00000000-0000-7000-8000-000000000104";
const WALLET_CENTER_MENU_ID: &str = "00000000-0000-7000-8000-000000000208";
const WALLET_BALANCE_API_ID: &str = "00000000-0000-7000-8000-000000000337";
const WALLET_TRANSACTIONS_API_ID: &str = "00000000-0000-7000-8000-000000000338";
const USER_ROLE: &str = constants::auth::DEFAULT_USER_ROLE;
const WALLET_BALANCE_CODE: &str = "wallet_balance_read";
const WALLET_TRANSACTIONS_CODE: &str = "wallet_transactions_read";

pub async fn seed_wallet_rbac(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let tx = manager.get_connection().begin().await?;
    insert_account_section(&tx).await?;
    insert_wallet_menu_item(&tx).await?;
    insert_wallet_api_permissions(&tx).await?;
    insert_wallet_role_bindings(&tx).await?;
    tx.commit().await
}

pub async fn delete_wallet_rbac(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let tx = manager.get_connection().begin().await?;
    delete_wallet_role_bindings(&tx).await?;
    delete_wallet_menu_item(&tx).await?;
    delete_wallet_api_permissions(&tx).await?;
    delete_account_section_if_empty(&tx).await?;
    tx.commit().await
}

async fn insert_account_section(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        INSERT INTO menu_sections (id, code, subheader, sort_order, enabled, created_at, updated_at)
        VALUES ('{ACCOUNT_SECTION_ID}', 'account', 'Account', -3, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (id) DO UPDATE
            SET code = EXCLUDED.code,
                subheader = EXCLUDED.subheader,
                sort_order = EXCLUDED.sort_order,
                enabled = EXCLUDED.enabled,
                updated_at = CURRENT_TIMESTAMP
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn insert_wallet_menu_item(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        INSERT INTO menu_items (
            id, section_id, parent_id, code, title, route_path, icon, caption,
            deep_match, sort_order, enabled, created_at, updated_at
        )
        VALUES (
            '{WALLET_CENTER_MENU_ID}', '{ACCOUNT_SECTION_ID}', NULL, 'wallet_center',
            'Wallet Center', '/dashboard/wallet', 'icon.wallet', NULL,
            TRUE, 0, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        ON CONFLICT (id) DO UPDATE
            SET section_id = EXCLUDED.section_id,
                parent_id = EXCLUDED.parent_id,
                code = EXCLUDED.code,
                title = EXCLUDED.title,
                route_path = EXCLUDED.route_path,
                icon = EXCLUDED.icon,
                caption = EXCLUDED.caption,
                deep_match = EXCLUDED.deep_match,
                sort_order = EXCLUDED.sort_order,
                enabled = EXCLUDED.enabled,
                updated_at = CURRENT_TIMESTAMP
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn insert_wallet_api_permissions(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    for permission in wallet_api_permissions() {
        insert_wallet_api_permission(tx, permission).await?;
    }
    Ok(())
}

async fn insert_wallet_api_permission(tx: &DatabaseTransaction, permission: WalletApiPermission) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        INSERT INTO api_permissions (id, code, method, path_pattern, name, "group", enabled, system, created_at, updated_at)
        VALUES (
            '{}', '{}', '{}', '{}', '{}', '{}',
            TRUE, TRUE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        ON CONFLICT (id) DO UPDATE
            SET code = EXCLUDED.code,
                method = EXCLUDED.method,
                path_pattern = EXCLUDED.path_pattern,
                name = EXCLUDED.name,
                "group" = EXCLUDED."group",
                enabled = EXCLUDED.enabled,
                system = EXCLUDED.system,
                updated_at = CURRENT_TIMESTAMP
        "#,
        permission.id, permission.code, permission.method, permission.path_pattern, permission.name, permission.group
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn insert_wallet_role_bindings(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    insert_role_api_binding(tx, USER_ROLE, WALLET_BALANCE_API_ID).await?;
    insert_role_api_binding(tx, USER_ROLE, WALLET_TRANSACTIONS_API_ID).await?;
    insert_role_menu_binding(tx, USER_ROLE, WALLET_CENTER_MENU_ID).await
}

async fn insert_role_api_binding(tx: &DatabaseTransaction, role_code: &str, api_id: &str) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        INSERT INTO role_api_permissions (role_code, api_permission_id, created_at, updated_at)
        VALUES ('{role_code}', '{api_id}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (role_code, api_permission_id) DO UPDATE
            SET updated_at = CURRENT_TIMESTAMP
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn insert_role_menu_binding(tx: &DatabaseTransaction, role_code: &str, menu_id: &str) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        INSERT INTO role_menu_permissions (role_code, menu_item_id, created_at, updated_at)
        VALUES ('{role_code}', '{menu_id}', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT (role_code, menu_item_id) DO UPDATE
            SET updated_at = CURRENT_TIMESTAMP
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn delete_wallet_role_bindings(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        DELETE FROM role_api_permissions
        WHERE api_permission_id IN ('{WALLET_BALANCE_API_ID}', '{WALLET_TRANSACTIONS_API_ID}');
        DELETE FROM role_menu_permissions
        WHERE menu_item_id = '{WALLET_CENTER_MENU_ID}';
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn delete_wallet_menu_item(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!("DELETE FROM menu_items WHERE id = '{WALLET_CENTER_MENU_ID}'");
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn delete_wallet_api_permissions(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        DELETE FROM api_permissions
        WHERE id IN ('{WALLET_BALANCE_API_ID}', '{WALLET_TRANSACTIONS_API_ID}')
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

async fn delete_account_section_if_empty(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    let sql = format!(
        r#"
        DELETE FROM menu_sections
        WHERE id = '{ACCOUNT_SECTION_ID}'
          AND NOT EXISTS (
              SELECT 1 FROM menu_items WHERE menu_items.section_id = menu_sections.id
          )
        "#
    );
    tx.execute_unprepared(&sql).await.map(|_| ())
}

fn wallet_api_permissions() -> [WalletApiPermission; 2] {
    [
        WalletApiPermission {
            id: WALLET_BALANCE_API_ID,
            code: WALLET_BALANCE_CODE,
            method: "GET",
            path_pattern: "/api/wallet/balance",
            name: "Wallet balance",
            group: "Wallet",
        },
        WalletApiPermission {
            id: WALLET_TRANSACTIONS_API_ID,
            code: WALLET_TRANSACTIONS_CODE,
            method: "GET",
            path_pattern: "/api/wallet/transactions",
            name: "Wallet transactions",
            group: "Wallet",
        },
    ]
}

struct WalletApiPermission {
    id: &'static str,
    code: &'static str,
    method: &'static str,
    path_pattern: &'static str,
    name: &'static str,
    group: &'static str,
}
