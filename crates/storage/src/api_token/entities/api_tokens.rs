use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::api_token::{ApiToken, ApiTokenType, ModelAccessMode};

use crate::user::UserEntity as Users;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: Option<String>,
    pub token_type: String,
    pub name: String,
    pub token_value: String,
    #[sea_orm(unique)]
    pub token_hash: String,
    pub token_prefix: String,
    pub group_code: String,
    pub expires_at: Option<TimeDateTimeWithTimeZone>,
    pub model_access_mode: String,
    pub allowed_model_ids: String,
    pub rate_limit_rpm: Option<i64>,
    pub quota_limit: Option<Decimal>,
    pub used_quota: Decimal,
    pub request_count: i64,
    pub is_active: bool,
    pub last_used_at: Option<TimeDateTimeWithTimeZone>,
    pub created_at: TimeDateTimeWithTimeZone,
    pub updated_at: TimeDateTimeWithTimeZone,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Users", from = "Column::UserId", to = "crate::user::UserColumn::Id")]
    User,
}

impl ActiveModelBehavior for ActiveModel {}

impl Related<Users> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Model {
    pub fn into_domain(self) -> crate::StorageResult<ApiToken> {
        Ok(ApiToken {
            id: self.id,
            user_id: self.user_id,
            token_type: token_type(&self.token_type)?,
            name: self.name,
            token_value: self.token_value,
            token_hash: self.token_hash,
            token_prefix: self.token_prefix,
            group_code: self.group_code,
            expires_at: self.expires_at.map(format_timestamp),
            model_access_mode: model_access_mode(&self.model_access_mode)?,
            allowed_model_ids: serde_json::from_str(&self.allowed_model_ids)?,
            rate_limit_rpm: self.rate_limit_rpm,
            quota_limit: self.quota_limit,
            used_quota: self.used_quota,
            request_count: self.request_count,
            is_active: self.is_active,
            last_used_at: self.last_used_at.map(format_timestamp),
            created_at: format_timestamp(self.created_at),
            updated_at: format_timestamp(self.updated_at),
        })
    }
}

pub(crate) fn token_type_value(value: ApiTokenType) -> &'static str {
    match value {
        ApiTokenType::User => "user",
        ApiTokenType::Independent => "independent",
    }
}

pub(crate) fn model_access_mode_value(value: ModelAccessMode) -> &'static str {
    match value {
        ModelAccessMode::All => "all",
        ModelAccessMode::Limited => "limited",
    }
}

fn token_type(value: &str) -> crate::StorageResult<ApiTokenType> {
    match value {
        "user" => Ok(ApiTokenType::User),
        "independent" => Ok(ApiTokenType::Independent),
        other => Err(crate::StorageError::Database(format!("invalid api token type: {other}"))),
    }
}

fn model_access_mode(value: &str) -> crate::StorageResult<ModelAccessMode> {
    match value {
        "all" => Ok(ModelAccessMode::All),
        "limited" => Ok(ModelAccessMode::Limited),
        other => Err(crate::StorageError::Database(format!("invalid model_access_mode: {other}"))),
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("api token timestamp must format as RFC3339")
}
