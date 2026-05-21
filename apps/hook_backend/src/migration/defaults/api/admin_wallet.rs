use super::ApiDefinition;

pub const ADMIN_WALLET_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "admin_wallet_user_balance_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/users/{user_id}/balance",
        name: "用户钱包余额",
    },
    ApiDefinition {
        code: "admin_wallets_read",
        method: "GET",
        path_pattern: "/api/admin/wallets",
        name: "钱包列表",
    },
    ApiDefinition {
        code: "admin_wallet_transactions_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/{id}/transactions",
        name: "钱包流水列表",
    },
    ApiDefinition {
        code: "admin_wallet_ledger_entries_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/{id}/ledger-entries",
        name: "钱包聚合流水列表",
    },
    ApiDefinition {
        code: "admin_wallet_daily_model_usage_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/{id}/ledger-entries/daily-model-usage",
        name: "钱包模型消费明细",
    },
    ApiDefinition {
        code: "admin_wallet_adjust",
        method: "POST",
        path_pattern: "/api/admin/wallets/{id}/adjust",
        name: "钱包调账",
    },
    ApiDefinition {
        code: "admin_wallet_recharge",
        method: "POST",
        path_pattern: "/api/admin/wallets/{id}/recharge",
        name: "钱包人工充值",
    },
    ApiDefinition {
        code: "admin_wallet_ledger_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/ledger",
        name: "全局资金流水",
    },
    ApiDefinition {
        code: "admin_wallet_ledger_entries_global_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/ledger-entries",
        name: "全局聚合资金流水",
    },
];
