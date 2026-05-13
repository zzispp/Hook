use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum Roles {
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
pub(in crate::migration::baseline) enum ApiPermissions {
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
pub(in crate::migration::baseline) enum MenuSections {
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
pub(in crate::migration::baseline) enum MenuItems {
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
pub(in crate::migration::baseline) enum MenuApiPermissions {
    Table,
    MenuItemId,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum RoleMenuPermissions {
    Table,
    RoleCode,
    MenuItemId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum RoleApiPermissions {
    Table,
    RoleCode,
    ApiPermissionId,
    CreatedAt,
    UpdatedAt,
}
