use sea_orm_migration::prelude::*;

mod auth;
mod billing;
mod provider;
mod rbac;
mod request;
mod settings;
mod token;
mod translation;
mod wallet;

pub(super) use auth::*;
pub(super) use billing::*;
pub(super) use provider::*;
pub(super) use rbac::*;
pub(super) use request::*;
pub(super) use settings::*;
pub(super) use token::*;
pub(super) use translation::*;
pub(super) use wallet::*;

pub fn reversed_tables() -> Vec<DynIden> {
    vec![
        RequestCandidates::Table.into_iden(),
        ApiTokens::Table.into_iden(),
        BillingGroupProviders::Table.into_iden(),
        BillingGroupModels::Table.into_iden(),
        ProviderModels::Table.into_iden(),
        ProviderApiKeys::Table.into_iden(),
        ProviderEndpoints::Table.into_iden(),
        TranslationEntries::Table.into_iden(),
        TranslationLanguages::Table.into_iden(),
        SystemSettings::Table.into_iden(),
        Providers::Table.into_iden(),
        GlobalModels::Table.into_iden(),
        BillingGroups::Table.into_iden(),
        WalletTransactions::Table.into_iden(),
        Wallets::Table.into_iden(),
        RoleApiPermissions::Table.into_iden(),
        RoleMenuPermissions::Table.into_iden(),
        MenuApiPermissions::Table.into_iden(),
        MenuItems::Table.into_iden(),
        MenuSections::Table.into_iden(),
        ApiPermissions::Table.into_iden(),
        Roles::Table.into_iden(),
        Users::Table.into_iden(),
    ]
}
