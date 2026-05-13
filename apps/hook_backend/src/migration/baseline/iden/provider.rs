use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum GlobalModels {
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
pub(in crate::migration::baseline) enum Providers {
    Table,
    Id,
    Name,
    ProviderType,
    MaxRetries,
    RequestTimeoutSeconds,
    StreamFirstByteTimeoutSeconds,
    Priority,
    KeepPriorityOnConversion,
    EnableFormatConversion,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderEndpoints {
    Table,
    Id,
    ProviderId,
    ApiFormat,
    BaseUrl,
    CustomPath,
    MaxRetries,
    IsActive,
    FormatAcceptanceConfig,
    HeaderRules,
    BodyRules,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderApiKeys {
    Table,
    Id,
    ProviderId,
    Name,
    EncryptedApiKey,
    Note,
    InternalPriority,
    RpmLimit,
    LearnedRpmLimit,
    CacheTtlMinutes,
    MaxProbeIntervalMinutes,
    TimeRangeEnabled,
    TimeRangeStart,
    TimeRangeEnd,
    HealthByFormat,
    CircuitBreakerByFormat,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderModels {
    Table,
    Id,
    ProviderId,
    GlobalModelId,
    ProviderModelName,
    ProviderModelMappings,
    IsActive,
    PricePerRequest,
    TieredPricing,
    Config,
    CreatedAt,
    UpdatedAt,
}
