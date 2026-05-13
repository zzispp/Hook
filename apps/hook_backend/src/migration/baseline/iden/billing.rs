use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroups {
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
pub(in crate::migration::baseline) enum BillingGroupModels {
    Table,
    Id,
    GroupCode,
    GlobalModelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroupProviders {
    Table,
    Id,
    GroupCode,
    ProviderId,
    CreatedAt,
    UpdatedAt,
}
