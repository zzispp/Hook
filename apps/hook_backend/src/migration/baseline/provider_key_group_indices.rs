use sea_orm_migration::prelude::*;

use super::iden::*;

pub(super) fn provider_key_group_indices() -> Vec<IndexCreateStatement> {
    vec![
        index("index_provider_key_groups_by_name", ProviderKeyGroups::Table, ProviderKeyGroups::Name, true),
        provider_key_group_keys_unique_index(),
        index(
            "index_provider_key_group_keys_by_key",
            ProviderKeyGroupKeys::Table,
            ProviderKeyGroupKeys::ProviderKeyId,
            false,
        ),
        billing_group_provider_key_groups_unique_index(),
        index(
            "index_billing_group_provider_key_groups_by_group",
            BillingGroupProviderKeyGroups::Table,
            BillingGroupProviderKeyGroups::GroupCode,
            false,
        ),
    ]
}

fn provider_key_group_keys_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_provider_key_group_keys_unique")
        .table(ProviderKeyGroupKeys::Table)
        .col(ProviderKeyGroupKeys::ProviderKeyGroupId)
        .col(ProviderKeyGroupKeys::ProviderKeyId)
        .unique()
        .if_not_exists()
        .to_owned()
}

fn billing_group_provider_key_groups_unique_index() -> IndexCreateStatement {
    Index::create()
        .name("index_billing_group_provider_key_groups_unique")
        .table(BillingGroupProviderKeyGroups::Table)
        .col(BillingGroupProviderKeyGroups::GroupCode)
        .col(BillingGroupProviderKeyGroups::ProviderKeyGroupId)
        .unique()
        .if_not_exists()
        .to_owned()
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
