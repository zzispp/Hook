use sea_orm_migration::{prelude::*, schema::*};

use super::iden::{TranslationEntries, TranslationLanguages};

pub(super) fn translation_languages_table() -> TableCreateStatement {
    Table::create()
        .table(TranslationLanguages::Table)
        .if_not_exists()
        .col(string_len(TranslationLanguages::Code, 32).primary_key())
        .col(string_len(TranslationLanguages::Name, 120))
        .col(string_len(TranslationLanguages::NativeName, 120))
        .col(boolean(TranslationLanguages::Enabled))
        .col(boolean(TranslationLanguages::System))
        .col(big_integer(TranslationLanguages::SortOrder))
        .col(timestamp_tz(TranslationLanguages::CreatedAt))
        .col(timestamp_tz(TranslationLanguages::UpdatedAt))
        .to_owned()
}

pub(super) fn translation_entries_table() -> TableCreateStatement {
    let mut language_fk = translation_language_fk();
    Table::create()
        .table(TranslationEntries::Table)
        .if_not_exists()
        .col(string_len(TranslationEntries::Id, 36).primary_key())
        .col(string_len(TranslationEntries::Namespace, 64))
        .col(string_len(TranslationEntries::GroupKey, 120))
        .col(string_len(TranslationEntries::ItemKey, 120))
        .col(string_len(TranslationEntries::LangCode, 32))
        .col(text(TranslationEntries::Value))
        .col(text_null(TranslationEntries::Description))
        .col(boolean(TranslationEntries::Enabled))
        .col(timestamp_tz(TranslationEntries::CreatedAt))
        .col(timestamp_tz(TranslationEntries::UpdatedAt))
        .foreign_key(&mut language_fk)
        .to_owned()
}

fn translation_language_fk() -> ForeignKeyCreateStatement {
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name("fk_translation_entries_language")
        .from(TranslationEntries::Table, TranslationEntries::LangCode)
        .to(TranslationLanguages::Table, TranslationLanguages::Code)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
