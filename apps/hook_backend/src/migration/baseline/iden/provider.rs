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
pub(in crate::migration::baseline) enum GlobalModelUserUsageCounts {
    Table,
    UserId,
    GlobalModelId,
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
    ProviderOrigin,
    MaxRetries,
    RequestTimeoutSeconds,
    StreamFirstByteTimeoutSeconds,
    StreamIdleTimeoutSeconds,
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
    ApiFormats,
    AllowedModelIds,
    Capabilities,
    EncryptedApiKey,
    Note,
    InternalPriority,
    GlobalPriorityByFormat,
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
pub(in crate::migration::baseline) enum ProviderKeyGroups {
    Table,
    Id,
    Name,
    Description,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderKeyGroupKeys {
    Table,
    Id,
    ProviderKeyGroupId,
    ProviderKeyId,
    Priority,
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
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderModelCosts {
    Table,
    Id,
    ProviderId,
    KeyId,
    ProviderModelId,
    CostMode,
    PricePerRequest,
    InputPricePerMillion,
    OutputPricePerMillion,
    CacheCreationPricePerMillion,
    CacheReadPricePerMillion,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderQuickImportSources {
    Table,
    Id,
    ProviderId,
    SourceKind,
    BaseUrl,
    EncryptedSystemAccessToken,
    UserId,
    RechargeMultiplier,
    AutoSyncEnabled,
    CostSyncMode,
    UpstreamAnomalyAction,
    TokenDeletedAction,
    TokenDisabledAction,
    GroupRemovedAction,
    GroupChangedAction,
    KeyUnavailableAction,
    ModelRemovedAction,
    FetchFailureAction,
    FetchFailureDisableThreshold,
    LastStatus,
    LastError,
    LastSyncedAt,
    ConsecutiveFailures,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderQuickImportKeys {
    Table,
    Id,
    ProviderId,
    SourceId,
    KeyId,
    UpstreamTokenId,
    UpstreamTokenName,
    UpstreamMaskedKey,
    UpstreamGroup,
    UpstreamGroupRatio,
    EffectiveCostMultiplier,
    SyncStatuses,
    LastSyncError,
    LastSyncedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderQuickImportKeyModels {
    Table,
    Id,
    ProviderId,
    SourceId,
    KeyId,
    UpstreamModelId,
    GlobalModelId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderQuickImportSyncEvents {
    Table,
    Id,
    ProviderId,
    SourceId,
    KeyId,
    Status,
    Title,
    Detail,
    CreatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderCooldowns {
    Table,
    ProviderId,
    ProviderNameSnapshot,
    StatusCode,
    ObservedCount,
    ThresholdCount,
    WindowSeconds,
    CooldownSeconds,
    TriggeredAt,
    CooldownUntil,
    ReleasedAt,
    RequestId,
    CandidateIndex,
    RetryIndex,
    EndpointId,
    EndpointNameSnapshot,
    KeyId,
    KeyNameSnapshot,
    ErrorType,
    ErrorMessage,
    ErrorCode,
    ErrorParam,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum ProviderCooldownEvents {
    Table,
    Id,
    ProviderId,
    ProviderNameSnapshot,
    StatusCode,
    ObservedCount,
    ThresholdCount,
    WindowSeconds,
    CooldownSeconds,
    TriggeredAt,
    CooldownUntil,
    RequestId,
    CandidateIndex,
    RetryIndex,
    EndpointId,
    EndpointNameSnapshot,
    KeyId,
    KeyNameSnapshot,
    ErrorType,
    ErrorMessage,
    ErrorCode,
    ErrorParam,
    CreatedAt,
}
