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
#[sea_orm(iden = "user_registration_email_verifications")]
pub(in crate::migration::baseline) enum ObsoleteUserRegistrationEmailVerifications {
    Table,
}
