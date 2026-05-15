use rust_decimal::Decimal;
use types::{
    api_token::{ApiTokenType, ModelAccessMode},
    model::PatchField,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ApiTokenRecordInput {
    pub user_id: Option<String>,
    pub token_type: ApiTokenType,
    pub name: String,
    pub token_value: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub group_code: String,
    pub expires_at: Option<time::OffsetDateTime>,
    pub model_access_mode: ModelAccessMode,
    pub allowed_model_ids: Vec<String>,
    pub rate_limit_rpm: Option<i64>,
    pub quota_limit: Option<Decimal>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApiTokenRecordPatch {
    pub name: Option<String>,
    pub group_code: Option<String>,
    pub expires_at: PatchField<time::OffsetDateTime>,
    pub model_access_mode: Option<ModelAccessMode>,
    pub allowed_model_ids: PatchField<Vec<String>>,
    pub rate_limit_rpm: PatchField<i64>,
    pub quota_limit: PatchField<Decimal>,
    pub is_active: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiTokenUsageRecord {
    pub token_id: String,
    pub cost: Decimal,
    pub request_count: i64,
    pub used_at: time::OffsetDateTime,
}
