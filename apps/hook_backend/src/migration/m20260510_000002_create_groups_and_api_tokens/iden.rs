use sea_orm_migration::prelude::*;

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
pub(super) enum ApiPermissions {
    Table,
    Id,
    Code,
    Method,
    PathPattern,
    Name,
    Group,
    Enabled,
    System,
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

pub(super) fn reversed_tables() -> Vec<DynIden> {
    vec![ApiTokens::Table.into_iden(), BillingGroups::Table.into_iden()]
}
