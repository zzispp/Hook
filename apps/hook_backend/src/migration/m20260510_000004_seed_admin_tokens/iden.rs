use sea_orm_migration::prelude::*;

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
