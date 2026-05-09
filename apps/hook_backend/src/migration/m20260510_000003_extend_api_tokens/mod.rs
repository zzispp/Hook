use sea_orm_migration::prelude::*;

pub struct Migration;

#[derive(DeriveIden)]
enum ApiTokens {
    Table,
    TokenType,
    RequestCount,
}

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260510_000003_extend_api_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ApiTokens::Table)
                    .add_column_if_not_exists(ColumnDef::new(ApiTokens::TokenType).string_len(20).not_null().default("user"))
                    .add_column_if_not_exists(ColumnDef::new(ApiTokens::RequestCount).big_integer().not_null().default(0))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index_api_tokens_by_token_type")
                    .table(ApiTokens::Table)
                    .col(ApiTokens::TokenType)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("index_api_tokens_by_token_type")
                    .table(ApiTokens::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ApiTokens::Table)
                    .drop_column(ApiTokens::RequestCount)
                    .drop_column(ApiTokens::TokenType)
                    .to_owned(),
            )
            .await
    }
}
