use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum TranslationLanguages {
    Table,
    Code,
    Name,
    NativeName,
    Enabled,
    System,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub(in crate::migration::baseline) enum TranslationEntries {
    Table,
    Id,
    Namespace,
    GroupKey,
    ItemKey,
    LangCode,
    Value,
    Description,
    Enabled,
    CreatedAt,
    UpdatedAt,
}
