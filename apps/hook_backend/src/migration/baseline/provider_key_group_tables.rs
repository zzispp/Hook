use sea_orm_migration::{prelude::*, schema::*};

use super::iden::*;

pub(super) fn provider_key_group_tables() -> Vec<TableCreateStatement> {
    vec![
        provider_key_groups_table(),
        provider_key_group_keys_table(),
        billing_group_provider_key_groups_table(),
    ]
}

fn provider_key_groups_table() -> TableCreateStatement {
    Table::create()
        .table(ProviderKeyGroups::Table)
        .if_not_exists()
        .col(string_len(ProviderKeyGroups::Id, 36).primary_key())
        .col(string_len(ProviderKeyGroups::Name, 100))
        .col(text_null(ProviderKeyGroups::Description))
        .col(big_integer(ProviderKeyGroups::SortOrder).default(0))
        .col(timestamp_tz(ProviderKeyGroups::CreatedAt))
        .col(timestamp_tz(ProviderKeyGroups::UpdatedAt))
        .to_owned()
}

fn provider_key_group_keys_table() -> TableCreateStatement {
    let mut group_fk = provider_key_group_key_group_fk();
    let mut key_fk = provider_key_group_key_key_fk();
    Table::create()
        .table(ProviderKeyGroupKeys::Table)
        .if_not_exists()
        .col(string_len(ProviderKeyGroupKeys::Id, 36).primary_key())
        .col(string_len(ProviderKeyGroupKeys::ProviderKeyGroupId, 36))
        .col(string_len(ProviderKeyGroupKeys::ProviderKeyId, 36))
        .col(integer(ProviderKeyGroupKeys::Priority))
        .col(timestamp_tz(ProviderKeyGroupKeys::CreatedAt))
        .col(timestamp_tz(ProviderKeyGroupKeys::UpdatedAt))
        .foreign_key(&mut group_fk)
        .foreign_key(&mut key_fk)
        .to_owned()
}

fn billing_group_provider_key_groups_table() -> TableCreateStatement {
    let mut billing_group_fk = billing_provider_key_group_billing_group_fk();
    let mut provider_key_group_fk = billing_provider_key_group_key_group_fk();
    Table::create()
        .table(BillingGroupProviderKeyGroups::Table)
        .if_not_exists()
        .col(string_len(BillingGroupProviderKeyGroups::Id, 36).primary_key())
        .col(string_len(BillingGroupProviderKeyGroups::GroupCode, 64))
        .col(string_len(BillingGroupProviderKeyGroups::ProviderKeyGroupId, 36))
        .col(timestamp_tz(BillingGroupProviderKeyGroups::CreatedAt))
        .col(timestamp_tz(BillingGroupProviderKeyGroups::UpdatedAt))
        .foreign_key(&mut billing_group_fk)
        .foreign_key(&mut provider_key_group_fk)
        .to_owned()
}

fn provider_key_group_key_group_fk() -> ForeignKeyCreateStatement {
    foreign_key(
        "fk_provider_key_group_keys_group",
        ProviderKeyGroupKeys::Table,
        ProviderKeyGroupKeys::ProviderKeyGroupId,
        ProviderKeyGroups::Table,
        ProviderKeyGroups::Id,
    )
}

fn provider_key_group_key_key_fk() -> ForeignKeyCreateStatement {
    foreign_key(
        "fk_provider_key_group_keys_key",
        ProviderKeyGroupKeys::Table,
        ProviderKeyGroupKeys::ProviderKeyId,
        ProviderApiKeys::Table,
        ProviderApiKeys::Id,
    )
}

fn billing_provider_key_group_billing_group_fk() -> ForeignKeyCreateStatement {
    foreign_key(
        "fk_billing_group_provider_key_groups_group",
        BillingGroupProviderKeyGroups::Table,
        BillingGroupProviderKeyGroups::GroupCode,
        BillingGroups::Table,
        BillingGroups::Code,
    )
}

fn billing_provider_key_group_key_group_fk() -> ForeignKeyCreateStatement {
    foreign_key(
        "fk_billing_group_provider_key_groups_key_group",
        BillingGroupProviderKeyGroups::Table,
        BillingGroupProviderKeyGroups::ProviderKeyGroupId,
        ProviderKeyGroups::Table,
        ProviderKeyGroups::Id,
    )
}

fn foreign_key<T, C, RT, RC>(name: &str, table: T, column: C, ref_table: RT, ref_column: RC) -> ForeignKeyCreateStatement
where
    T: IntoIden,
    C: IntoIden,
    RT: IntoIden,
    RC: IntoIden,
{
    let mut foreign_key = ForeignKey::create();
    foreign_key
        .name(name)
        .from(table, column)
        .to(ref_table, ref_column)
        .on_delete(ForeignKeyAction::Cascade);
    foreign_key
}

fn timestamp_tz<T>(col: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(col).timestamp_with_time_zone().not_null().take()
}
