use sea_orm::entity::prelude::*;
use time::format_description::well_known::Rfc3339;
use types::recharge::PaymentCallbackRecord;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "payment_callback_records")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub payment_channel_code: String,
    pub callback_kind: String,
    pub http_method: String,
    pub order_no: Option<String>,
    pub provider_trade_no: Option<String>,
    pub payment_method: Option<String>,
    pub trade_status: Option<String>,
    pub status: String,
    pub settled: bool,
    pub error_message: Option<String>,
    pub raw_params_json: String,
    pub received_at: TimeDateTimeWithTimeZone,
    pub processed_at: Option<TimeDateTimeWithTimeZone>,
}

#[derive(Clone, Copy, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for PaymentCallbackRecord {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            payment_channel_code: value.payment_channel_code,
            callback_kind: value.callback_kind,
            http_method: value.http_method,
            order_no: value.order_no,
            provider_trade_no: value.provider_trade_no,
            payment_method: value.payment_method,
            trade_status: value.trade_status,
            status: value.status,
            settled: value.settled,
            error_message: value.error_message,
            raw_params: serde_json::from_str(&value.raw_params_json).unwrap_or_else(|_| serde_json::json!({})),
            received_at: format_timestamp(value.received_at),
            processed_at: value.processed_at.map(format_timestamp),
        }
    }
}

fn format_timestamp(value: TimeDateTimeWithTimeZone) -> String {
    value.format(&Rfc3339).expect("payment callback timestamp must format as RFC3339")
}
