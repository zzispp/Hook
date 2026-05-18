use async_trait::async_trait;
use storage::{
    Database, StorageError,
    card_code::{CardCodeStore, CardCodeTypeRecordInput, CardCodeTypeRecordPatch},
};
use types::{
    card_code::{
        CardCode, CardCodeCreateRecord, CardCodeListFilters, CardCodeRedeemInput, CardCodeRedeemResponse, CardCodeType, CardCodeTypeCreatePayload,
        CardCodeTypeListFilters, CardCodeTypeUpdatePayload,
    },
    pagination::{Page, PageRequest, PageSliceRequest},
};

use crate::application::{CardCodeError, CardCodeRepository, CardCodeResult};

#[derive(Clone)]
pub struct StorageCardCodeRepository {
    store: CardCodeStore,
}

impl StorageCardCodeRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: CardCodeStore::new(database),
        }
    }
}

#[async_trait]
impl CardCodeRepository for StorageCardCodeRepository {
    async fn list_types(&self, page: PageRequest, filters: CardCodeTypeListFilters) -> CardCodeResult<Page<CardCodeType>> {
        self.store.list_types(page_slice_request(page), filters).await.map_err(storage_error)
    }

    async fn create_type(&self, input: CardCodeTypeCreatePayload) -> CardCodeResult<CardCodeType> {
        self.store.create_type(type_input(input)).await.map_err(storage_error)
    }

    async fn update_type(&self, id: &str, input: CardCodeTypeUpdatePayload) -> CardCodeResult<CardCodeType> {
        self.store.update_type(id, type_patch(input)).await.map_err(storage_error)
    }

    async fn find_type(&self, id: &str) -> CardCodeResult<Option<CardCodeType>> {
        self.store.find_type(id).await.map_err(storage_error)
    }

    async fn code_exists(&self, code: &str) -> CardCodeResult<bool> {
        self.store.code_exists(code).await.map_err(storage_error)
    }

    async fn create_codes(&self, inputs: Vec<CardCodeCreateRecord>) -> CardCodeResult<Vec<CardCode>> {
        self.store.create_codes(inputs).await.map_err(storage_error)
    }

    async fn list_codes(&self, page: PageRequest, filters: CardCodeListFilters) -> CardCodeResult<Page<CardCode>> {
        self.store.list_codes(page_slice_request(page), filters).await.map_err(storage_error)
    }

    async fn batch_update_code_status(&self, ids: &[String], status: &str) -> CardCodeResult<u64> {
        self.store.batch_update_code_status(ids, status).await.map_err(storage_error)
    }

    async fn redeem(&self, input: CardCodeRedeemInput) -> CardCodeResult<CardCodeRedeemResponse> {
        self.store.redeem(input).await.map(Into::into).map_err(storage_error)
    }
}

fn type_input(input: CardCodeTypeCreatePayload) -> CardCodeTypeRecordInput {
    CardCodeTypeRecordInput {
        name: input.name,
        balance_type: input.balance_type,
        status: input.status.unwrap_or_else(|| types::card_code::CARD_CODE_STATUS_ACTIVE.into()),
        remark: input.remark,
    }
}

fn type_patch(input: CardCodeTypeUpdatePayload) -> CardCodeTypeRecordPatch {
    CardCodeTypeRecordPatch {
        name: input.name,
        balance_type: input.balance_type,
        status: input.status,
        remark: input.remark,
    }
}

fn storage_error(error: StorageError) -> CardCodeError {
    match error {
        StorageError::NotFound => CardCodeError::NotFound,
        StorageError::Conflict(message) => CardCodeError::Conflict(message),
        StorageError::Database(message) => CardCodeError::Infrastructure(message),
    }
}

fn page_slice_request(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}
