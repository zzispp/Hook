use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    Email,
    Role,
    IsActive,
    IsDeleted,
    AllowedModelIds,
    AllowedProviderIds,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
    AuthSource,
    EmailVerified,
    RateLimitRpm,
    QuotaMode,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum UserGroupMemberships {
    Table,
    Id,
    UserId,
    UserGroupCode,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum UserGroups {
    Table,
    Id,
    Code,
    Name,
    Description,
    IsActive,
    IsSystem,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum UserPasswordResetTokens {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    ConsumedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum UserIdentities {
    Table,
    Id,
    UserId,
    Provider,
    ProviderSubject,
    Email,
    EmailVerified,
    DisplayName,
    AvatarUrl,
    MetadataJson,
    CreatedAt,
    UpdatedAt,
    LastLoginAt,
}
