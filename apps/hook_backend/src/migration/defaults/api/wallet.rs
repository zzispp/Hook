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
];
