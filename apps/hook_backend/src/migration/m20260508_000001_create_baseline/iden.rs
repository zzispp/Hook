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
}

#[derive(DeriveIden)]
pub(super) enum MenuSections {
    Table,
    Id,
    Code,
    Subheader,
    SortOrder,
    Enabled,
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
}

#[derive(DeriveIden)]
pub(super) enum RoleApiPermissions {
    Table,
    RoleCode,
    ApiPermissionId,
}

#[derive(DeriveIden)]
pub(super) enum RoleMenuPermissions {
    Table,
    RoleCode,
    MenuItemId,
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

pub(super) fn reversed_tables() -> Vec<DynIden> {
    vec![
        Models::Table.into_iden(),
        GlobalModels::Table.into_iden(),
        RoleMenuPermissions::Table.into_iden(),
        RoleApiPermissions::Table.into_iden(),
        MenuItems::Table.into_iden(),
        MenuSections::Table.into_iden(),
        ApiPermissions::Table.into_iden(),
        Roles::Table.into_iden(),
        Users::Table.into_iden(),
    ]
}
