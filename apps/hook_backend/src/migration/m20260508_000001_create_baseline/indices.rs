use sea_orm_migration::prelude::*;

use super::iden::*;

pub(super) fn baseline_indices() -> Vec<IndexCreateStatement> {
    vec![
        index("index_users_by_username", Users::Table, Users::Username, true),
        index("index_users_by_email", Users::Table, Users::Email, true),
        index("index_api_permissions_by_code", ApiPermissions::Table, ApiPermissions::Code, true),
        index("index_menu_sections_by_code", MenuSections::Table, MenuSections::Code, true),
        index("index_menu_items_by_section_id", MenuItems::Table, MenuItems::SectionId, false),
        index("index_menu_items_by_code", MenuItems::Table, MenuItems::Code, true),
        index("index_global_models_by_name", GlobalModels::Table, GlobalModels::Name, true),
        index("index_global_models_by_usage_count", GlobalModels::Table, GlobalModels::UsageCount, false),
        index("index_models_by_provider_id", Models::Table, Models::ProviderId, false),
        index("index_models_by_global_model_id", Models::Table, Models::GlobalModelId, false),
    ]
}

fn index<T, C>(name: &str, table: T, column: C, unique: bool) -> IndexCreateStatement
where
    T: Iden + 'static,
    C: Iden + 'static,
{
    let mut index = Index::create();
    index.name(name).table(table).col(column).if_not_exists();
    if unique {
        index.unique();
    }
    index.to_owned()
}
