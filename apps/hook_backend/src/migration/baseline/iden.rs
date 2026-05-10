use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(super) enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    Email,
    Role,
    IsActive,
    IsDeleted,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
    AuthSource,
    EmailVerified,
    RateLimitRpm,
    QuotaMode,
}

#[derive(DeriveIden)]
pub(super) enum Roles {
    Table,
    Code,
    Name,
    Description,
    Enabled,
    System,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum ApiPermissions {
    Table,
    Id,
    Code,
    Method,
    PathPattern,
    Name,
    Enabled,
    System,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum MenuSections {
    Table,
    Id,
    Code,
    Subheader,
    SortOrder,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum MenuItems {
    Table,
    Id,
    SectionId,
    ParentId,
    Code,
    Title,
    RoutePath,
    Icon,
    Caption,
    DeepMatch,
    SortOrder,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum MenuApiPermissions {
    Table,
    MenuItemId,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum RoleMenuPermissions {
    Table,
    RoleCode,
    MenuItemId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum RoleApiPermissions {
    Table,
    RoleCode,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum Wallets {
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
pub(super) enum WalletTransactions {
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

#[derive(DeriveIden)]
pub(super) enum GlobalModels {
    Table,
    Id,
    Name,
    DisplayName,
    DefaultPricePerRequest,
    DefaultTieredPricing,
    SupportedCapabilities,
    Config,
    IsActive,
    UsageCount,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum Models {
    Table,
    Id,
    ProviderId,
    GlobalModelId,
    ProviderModelName,
    ProviderModelMappings,
    PricePerRequest,
    TieredPricing,
    SupportsVision,
    SupportsFunctionCalling,
    SupportsStreaming,
    SupportsExtendedThinking,
    SupportsImageGeneration,
    IsActive,
    IsAvailable,
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum BillingGroups {
    Table,
    Id,
    Code,
    Name,
    Description,
    BillingMultiplier,
    IsActive,
    IsSystem,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum ApiTokens {
    Table,
    Id,
    UserId,
    TokenType,
    Name,
    TokenValue,
    TokenHash,
    TokenPrefix,
    GroupCode,
    ExpiresAt,
    ModelAccessMode,
    AllowedModelIds,
    RateLimitRpm,
    QuotaLimit,
    UsedQuota,
    RequestCount,
    IsActive,
    LastUsedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum BillingGroupModels {
    Table,
    Id,
    GroupCode,
    GlobalModelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(super) enum SystemSettings {
    Table,
    Id,
    SiteName,
    SiteSubtitle,
    AllowRegistration,
    AutoDeleteExpiredTokens,
    DefaultUserGrant,
    DefaultRateLimitRpm,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration) enum TranslationLanguages {
    Table,
    Code,
    Name,
    NativeName,
    Enabled,
    System,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration) enum TranslationEntries {
    Table,
    Id,
    Namespace,
    GroupKey,
    ItemKey,
    LangCode,
    Value,
    Description,
    Enabled,
    CreatedAt,
    UpdatedAt,
}

pub fn reversed_tables() -> Vec<DynIden> {
    vec![
        ApiTokens::Table.into_iden(),
        BillingGroupModels::Table.into_iden(),
        TranslationEntries::Table.into_iden(),
        TranslationLanguages::Table.into_iden(),
        SystemSettings::Table.into_iden(),
        Models::Table.into_iden(),
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
