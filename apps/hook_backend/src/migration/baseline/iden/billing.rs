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

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroupProviderKeys {
    Table,
    Id,
    GroupCode,
    ProviderKeyId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroupProviderGroups {
    Table,
    Id,
    GroupCode,
    ProviderGroupId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroupProviderKeyGroups {
    Table,
    Id,
    GroupCode,
    ProviderKeyGroupId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingGroupUserGroups {
    Table,
    Id,
    BillingGroupCode,
    UserGroupCode,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum BillingRules {
    Table,
    Id,
    GlobalModelId,
    ModelId,
    Name,
    TaskType,
    Expression,
    Variables,
    DimensionMappings,
    IsEnabled,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum DimensionCollectors {
    Table,
    Id,
    ApiFormat,
    TaskType,
    DimensionName,
    SourceType,
    SourcePath,
    ValueType,
    TransformExpression,
    DefaultValue,
    Priority,
    IsEnabled,
    CreatedAt,
    UpdatedAt,
}
