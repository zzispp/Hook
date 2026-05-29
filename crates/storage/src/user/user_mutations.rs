use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};

use crate::{StorageResult, api_token::api_token_records};

pub(super) async fn set_wallet_limit_mode(db: &sea_orm::DatabaseTransaction, user_id: &str, quota_mode: &str) -> StorageResult<()> {
    crate::wallet::wallet_records::Entity::update_many()
        .col_expr(crate::wallet::wallet_records::Column::LimitMode, Expr::value(wallet_limit_mode(quota_mode)))
        .filter(crate::wallet::wallet_records::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

pub(super) async fn delete_user_api_tokens(db: &sea_orm::DatabaseTransaction, user_id: &str) -> StorageResult<()> {
    api_token_records::Entity::delete_many()
        .filter(api_token_records::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

fn wallet_limit_mode(quota_mode: &str) -> &'static str {
    match quota_mode {
        types::user::USER_QUOTA_MODE_UNLIMITED => "unlimited",
        _ => "finite",
    }
}
