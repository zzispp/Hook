use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    pagination::Page,
    wallet::{WalletTransaction, WalletTransactionResponse},
};

pub const CARD_CODE_STATUS_ACTIVE: &str = "active";
pub const CARD_CODE_STATUS_DISABLED: &str = "disabled";
pub const CARD_CODE_STATUS_USED: &str = "used";
pub const CARD_CODE_STATUS_EXPIRED: &str = "expired";
pub const CARD_CODE_BALANCE_TYPE_RECHARGE: &str = "recharge";
pub const CARD_CODE_BALANCE_TYPE_GIFT: &str = "gift";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeType {
    pub id: String,
    pub name: String,
    pub balance_type: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCode {
    pub id: String,
    pub code: String,
    pub batch_no: String,
    pub type_id: String,
    pub type_name: String,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub currency: String,
    pub status: String,
    pub remark: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by_user_id: Option<String>,
    pub created_by_username: Option<String>,
    pub created_ip: Option<String>,
    pub used_by_user_id: Option<String>,
    pub used_by_username: Option<String>,
    pub used_ip: Option<String>,
    pub used_at: Option<String>,
    pub wallet_id: Option<String>,
    pub wallet_transaction_id: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardCodeTypeListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardCodeListFilters {
    pub search: Option<String>,
    pub status: Option<String>,
    pub type_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CardCodeTypeCreatePayload {
    pub name: String,
    pub balance_type: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CardCodeTypeUpdatePayload {
    pub name: String,
    pub balance_type: String,
    pub status: String,
    #[serde(default)]
    pub remark: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CardCodeGeneratePayload {
    pub type_id: String,
    pub quantity: u64,
    pub code_length: u8,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CardCodeBatchStatusPayload {
    pub ids: Vec<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CardCodeRedeemPayload {
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeCreateRecord {
    pub code: String,
    pub batch_no: String,
    pub type_id: String,
    pub type_name: String,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub currency: String,
    pub status: String,
    pub remark: Option<String>,
    pub expires_at: Option<String>,
    pub created_by_user_id: Option<String>,
    pub created_by_username: Option<String>,
    pub created_ip: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeRedeemInput {
    pub code: String,
    pub user_id: String,
    pub username: String,
    pub client_ip: Option<String>,
    pub target_currency: String,
    pub usd_cny_rate: Option<Decimal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeRedeemRecord {
    pub card_code: CardCode,
    pub transaction: WalletTransaction,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeTypeResponse {
    pub id: String,
    pub name: String,
    pub balance_type: String,
    pub status: String,
    pub remark: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeResponse {
    pub id: String,
    pub code: String,
    pub batch_no: String,
    pub type_id: String,
    pub type_name: String,
    #[serde(with = "rust_decimal::serde::float")]
    pub recharge_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub gift_amount: Decimal,
    pub currency: String,
    pub status: String,
    pub remark: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by_user_id: Option<String>,
    pub created_by_username: Option<String>,
    pub created_ip: Option<String>,
    pub used_by_user_id: Option<String>,
    pub used_by_username: Option<String>,
    pub used_ip: Option<String>,
    pub used_at: Option<String>,
    pub wallet_id: Option<String>,
    pub wallet_transaction_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeTypeListResponse {
    pub items: Vec<CardCodeTypeResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeListResponse {
    pub items: Vec<CardCodeResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeGenerateResponse {
    pub items: Vec<CardCodeResponse>,
    pub total: u64,
    pub batch_no: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CardCodeBatchStatusResponse {
    pub updated_count: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CardCodeRedeemResponse {
    pub card_code: CardCodeResponse,
    pub transaction: WalletTransactionResponse,
}

impl From<CardCodeType> for CardCodeTypeResponse {
    fn from(value: CardCodeType) -> Self {
        Self {
            id: value.id,
            name: value.name,
            balance_type: value.balance_type,
            status: value.status,
            remark: value.remark,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<CardCode> for CardCodeResponse {
    fn from(value: CardCode) -> Self {
        Self {
            status: effective_status(&value),
            id: value.id,
            code: value.code,
            batch_no: value.batch_no,
            type_id: value.type_id,
            type_name: value.type_name,
            recharge_amount: value.recharge_amount,
            gift_amount: value.gift_amount,
            currency: value.currency,
            remark: value.remark,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
            created_by_user_id: value.created_by_user_id,
            created_by_username: value.created_by_username,
            created_ip: value.created_ip,
            used_by_user_id: value.used_by_user_id,
            used_by_username: value.used_by_username,
            used_ip: value.used_ip,
            used_at: value.used_at,
            wallet_id: value.wallet_id,
            wallet_transaction_id: value.wallet_transaction_id,
        }
    }
}

impl From<Page<CardCodeType>> for CardCodeTypeListResponse {
    fn from(value: Page<CardCodeType>) -> Self {
        Self {
            items: value.items.into_iter().map(CardCodeTypeResponse::from).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<Page<CardCode>> for CardCodeListResponse {
    fn from(value: Page<CardCode>) -> Self {
        Self {
            items: value.items.into_iter().map(CardCodeResponse::from).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<CardCodeRedeemRecord> for CardCodeRedeemResponse {
    fn from(value: CardCodeRedeemRecord) -> Self {
        Self {
            card_code: value.card_code.into(),
            transaction: value.transaction.into(),
        }
    }
}

fn effective_status(value: &CardCode) -> String {
    if value.status == CARD_CODE_STATUS_ACTIVE && is_expired(value.expires_at.as_deref()) {
        return CARD_CODE_STATUS_EXPIRED.into();
    }
    value.status.clone()
}

fn is_expired(value: Option<&str>) -> bool {
    let Some(raw) = value else {
        return false;
    };
    let Ok(expires_at) = time::OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339) else {
        return false;
    };
    expires_at <= time::OffsetDateTime::now_utc()
}
