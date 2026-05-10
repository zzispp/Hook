use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::i18n::TranslationEntry;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "translation_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub namespace: String,
    pub group_key: String,
    pub item_key: String,
    pub lang_code: String,
    pub value: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for TranslationEntry {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            namespace: value.namespace,
            group_key: value.group_key,
            item_key: value.item_key,
            lang_code: value.lang_code,
            value: value.value,
            description: value.description,
            enabled: value.enabled,
            created_at: format_timestamp(value.created_at),
            updated_at: format_timestamp(value.updated_at),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("translation entry timestamp must format as RFC3339")
}
