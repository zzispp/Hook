use sea_orm_migration::prelude::*;

mod auth;
mod billing;
mod card_code;
mod operations;
mod performance_monitoring;
mod provider;
mod rbac;
mod recharge;
mod request;
mod scheduler;
mod settings;
mod token;
mod translation;
mod usage_flush;
mod wallet;

pub(super) use auth::*;
pub(super) use billing::*;
pub(super) use card_code::*;
pub(super) use operations::*;
pub(super) use performance_monitoring::*;
pub(super) use provider::*;
pub(super) use rbac::*;
pub(super) use recharge::*;
pub(super) use request::*;
pub(super) use scheduler::*;
pub(super) use settings::*;
pub(super) use token::*;
pub(super) use translation::*;
pub(super) use usage_flush::*;
pub(super) use wallet::*;

pub fn reversed_tables() -> Vec<DynIden> {
    vec![
        UsageFlushBatches::Table.into_iden(),
        PaymentCallbackRecords::Table.into_iden(),
        RechargeOrders::Table.into_iden(),
        RechargePackages::Table.into_iden(),
        PaymentChannels::Table.into_iden(),
        PerformanceMonitoringSnapshots::Table.into_iden(),
        NotificationStates::Table.into_iden(),
        ScheduledTaskRuns::Table.into_iden(),
        ScheduledTasks::Table.into_iden(),
        SupportTicketEmailEvents::Table.into_iden(),
        SupportTicketMessages::Table.into_iden(),
        SupportTickets::Table.into_iden(),
        Announcements::Table.into_iden(),
        DashboardCostAnalysisBuckets::Table.into_iden(),
        DashboardUserUsageBuckets::Table.into_iden(),
        RequestCandidates::Table.into_iden(),
        RequestRecords::Table.into_iden(),
        ApiTokens::Table.into_iden(),
        BillingGroupUserGroups::Table.into_iden(),
        DimensionCollectors::Table.into_iden(),
        BillingRules::Table.into_iden(),
        BillingGroupProviders::Table.into_iden(),
        BillingGroupModels::Table.into_iden(),
        ProviderModelCosts::Table.into_iden(),
        ProviderModels::Table.into_iden(),
        ProviderCooldowns::Table.into_iden(),
        ProviderApiKeys::Table.into_iden(),
        ProviderEndpoints::Table.into_iden(),
        TranslationEntries::Table.into_iden(),
        TranslationLanguages::Table.into_iden(),
        SystemSettings::Table.into_iden(),
        Providers::Table.into_iden(),
        GlobalModels::Table.into_iden(),
        BillingGroups::Table.into_iden(),
        CardCodes::Table.into_iden(),
        CardCodeTypes::Table.into_iden(),
        WalletTransactions::Table.into_iden(),
        Wallets::Table.into_iden(),
        RoleApiPermissions::Table.into_iden(),
        RoleMenuPermissions::Table.into_iden(),
        MenuApiPermissions::Table.into_iden(),
        MenuItems::Table.into_iden(),
        MenuSections::Table.into_iden(),
        ApiPermissions::Table.into_iden(),
        Roles::Table.into_iden(),
        UserPasswordResetTokens::Table.into_iden(),
        Users::Table.into_iden(),
        UserGroups::Table.into_iden(),
    ]
}
