use super::ApiDefinition;

pub const WALLET_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "wallet_balance_read",
        method: "GET",
        path_pattern: "/api/wallet/balance",
        name: "钱包余额",
    },
    ApiDefinition {
        code: "wallet_transactions_read",
        method: "GET",
        path_pattern: "/api/wallet/transactions",
        name: "钱包流水",
    },
    ApiDefinition {
        code: "wallet_ledger_entries_read",
        method: "GET",
        path_pattern: "/api/wallet/ledger-entries",
        name: "钱包聚合流水",
    },
    ApiDefinition {
        code: "wallet_daily_model_usage_read",
        method: "GET",
        path_pattern: "/api/wallet/ledger-entries/daily-model-usage",
        name: "钱包模型消费明细",
    },
];
