use super::ApiDefinition;

pub const ADMIN_WALLET_APIS: &[ApiDefinition] = &[
    ApiDefinition {
        code: "admin_wallets_read",
        method: "GET",
        path_pattern: "/api/admin/wallets",
        name: "List admin wallets",
        group: "Admin Wallet",
    },
    ApiDefinition {
        code: "admin_wallet_transactions_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/{id}/transactions",
        name: "Admin wallet transactions",
        group: "Admin Wallet",
    },
    ApiDefinition {
        code: "admin_wallet_adjust",
        method: "POST",
        path_pattern: "/api/admin/wallets/{id}/adjust",
        name: "Admin wallet adjustment",
        group: "Admin Wallet",
    },
    ApiDefinition {
        code: "admin_wallet_ledger_read",
        method: "GET",
        path_pattern: "/api/admin/wallets/ledger",
        name: "Admin wallet global ledger",
        group: "Admin Wallet",
    },
];
