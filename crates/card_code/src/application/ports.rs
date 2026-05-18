use async_trait::async_trait;
use types::{
    card_code::{
        CardCode, CardCodeBatchStatusPayload, CardCodeBatchStatusResponse, CardCodeCreateRecord, CardCodeGeneratePayload, CardCodeGenerateResponse,
        CardCodeListFilters, CardCodeListResponse, CardCodeRedeemInput, CardCodeRedeemPayload, CardCodeRedeemResponse, CardCodeType, CardCodeTypeCreatePayload,
        CardCodeTypeListFilters, CardCodeTypeListResponse, CardCodeTypeUpdatePayload,
    },
    pagination::{Page, PageRequest},
};

use super::CardCodeResult;

#[async_trait]
pub trait CardCodeRepository: Send + Sync + 'static {
    async fn list_types(&self, page: PageRequest, filters: CardCodeTypeListFilters) -> CardCodeResult<Page<CardCodeType>>;
    async fn create_type(&self, input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType>;
    async fn update_type(&self, id: &str, input: CardCodeTypeUpdatePayload) -> CardCodeResult<CardCodeType>;
    async fn find_type(&self, id: &str) -> CardCodeResult<Option<CardCodeType>>;
    async fn code_exists(&self, code: &str) -> CardCodeResult<bool>;
    async fn create_codes(&self, inputs: Vec<CardCodeCreateRecord>) -> CardCodeResult<Vec<CardCode>>;
    async fn list_codes(&self, page: PageRequest, filters: CardCodeListFilters) -> CardCodeResult<Page<CardCode>>;
    async fn batch_update_code_status(&self, ids: &[String], status: &str) -> CardCodeResult<u64>;
    async fn redeem(&self, input: CardCodeRedeemInput) -> CardCodeResult<CardCodeRedeemResponse>;
}

#[async_trait]
pub trait CardCodeUseCase: Send + Sync + 'static {
    async fn list_types(&self, page: PageRequest, filters: CardCodeTypeListFilters) -> CardCodeResult<CardCodeTypeListResponse>;
    async fn create_type(&self, input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType>;
    async fn update_type(&self, id: &str, input: CardCodeTypeUpdatePayload) -> CardCodeResult<CardCodeType>;
    async fn list_codes(&self, page: PageRequest, filters: CardCodeListFilters) -> CardCodeResult<CardCodeListResponse>;
    async fn generate_codes(&self, input: CardCodeGeneratePayload, operator: CardCodeOperator) -> CardCodeResult<CardCodeGenerateResponse>;
    async fn batch_update_code_status(&self, input: CardCodeBatchStatusPayload) -> CardCodeResult<CardCodeBatchStatusResponse>;
    async fn redeem(&self, input: CardCodeRedeemPayload, user: CardCodeRedeemer) -> CardCodeResult<CardCodeRedeemResponse>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeOperator {
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub client_ip: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardCodeRedeemer {
    pub user_id: String,
    pub username: String,
    pub client_ip: Option<String>,
}
