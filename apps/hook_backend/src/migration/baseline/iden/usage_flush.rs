use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum UsageFlushBatches {
    Table,
    Id,
    UsageKind,
    RecordCount,
    CreatedAt,
}
