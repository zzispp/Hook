use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::i18n::TranslationLanguage;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "translation_languages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for TranslationLanguage {
    fn from(value: Model) -> Self {
        Self {
            code: value.code,
            name: value.name,
            native_name: value.native_name,
            enabled: value.enabled,
            system: value.system,
            sort_order: value.sort_order,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("translation language timestamp must format as RFC3339")
}
