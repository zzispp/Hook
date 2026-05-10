use sea_orm_migration::prelude::*;

use super::iden::*;

const DEFAULT_GROUP_ID: &str = "00000000-0000-7000-8000-000000000401";
const SYSTEM_SETTINGS_ID: &str = "global";

pub(super) async fn seed_domain_defaults(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    seed_default_group(manager).await?;
    seed_system_settings(manager).await
}

async fn seed_default_group(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(BillingGroups::Table)
                .columns([
                    BillingGroups::Id,
                    BillingGroups::Code,
                    BillingGroups::Name,
                    BillingGroups::Description,
                    BillingGroups::BillingMultiplier,
                    BillingGroups::IsActive,
                    BillingGroups::IsSystem,
                    BillingGroups::SortOrder,
                    BillingGroups::CreatedAt,
                    BillingGroups::UpdatedAt,
                ])
                .values_panic([
                    DEFAULT_GROUP_ID.into(),
                    constants::billing::DEFAULT_SYSTEM_GROUP_CODE.into(),
                    "System Group".into(),
                    Some("Built-in billing group used when a token does not choose a group").into(),
                    1.into(),
                    true.into(),
                    true.into(),
                    0.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .to_owned(),
        )
        .await
}

async fn seed_system_settings(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    manager
        .execute(
            Query::insert()
                .into_table(SystemSettings::Table)
                .columns([
                    SystemSettings::Id,
                    SystemSettings::SiteName,
                    SystemSettings::SiteSubtitle,
                    SystemSettings::AllowRegistration,
                    SystemSettings::AutoDeleteExpiredTokens,
                    SystemSettings::DefaultUserGrant,
                    SystemSettings::DefaultRateLimitRpm,
                    SystemSettings::CreatedAt,
                    SystemSettings::UpdatedAt,
                ])
                .values_panic([
                    SYSTEM_SETTINGS_ID.into(),
                    "Hook".into(),
                    "AI API platform".into(),
                    true.into(),
                    false.into(),
                    0.into(),
                    0.into(),
                    Expr::current_timestamp(),
                    Expr::current_timestamp(),
                ])
                .to_owned(),
        )
        .await
}
