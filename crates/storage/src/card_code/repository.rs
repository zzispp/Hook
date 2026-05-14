use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    Set, sea_query::Expr,
};
use types::{
    card_code::{
        CARD_CODE_STATUS_ACTIVE, CARD_CODE_STATUS_DISABLED, CardCode, CardCodeCreateRecord,
        CardCodeListFilters, CardCodeRedeemInput, CardCodeRedeemRecord, CardCodeType,
        CardCodeTypeListFilters,
    },
    pagination::{Page, PageSliceRequest},
};

use crate::{
    Database, StorageError, StorageResult,
    card_code::{
        CardCodeTypeRecordInput, CardCodeTypeRecordPatch, card_code_records, card_code_type_records,
        query, redemption, time_format,
    },
};

#[derive(Clone)]
pub struct CardCodeStore {
    database: Database,
}

impl CardCodeStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create_type(&self, input: CardCodeTypeRecordInput) -> StorageResult<CardCodeType> {
        let now = time::OffsetDateTime::now_utc();
        let record = card_code_type_records::ActiveModel {
            id: Set(self.database.next_id()),
            name: Set(input.name),
            balance_type: Set(input.balance_type),
            status: Set(input.status),
            remark: Set(input.remark),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await?;
        Ok(record.into())
    }

    pub async fn update_type(
        &self,
        id: &str,
        input: CardCodeTypeRecordPatch,
    ) -> StorageResult<CardCodeType> {
        let record = self.find_type_record(id).await?.ok_or(StorageError::NotFound)?;
        let mut active: card_code_type_records::ActiveModel = record.into();
        active.name = Set(input.name);
        active.balance_type = Set(input.balance_type);
        active.status = Set(input.status);
        active.remark = Set(input.remark);
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        Ok(active.update(self.database.connection()).await?.into())
    }

    pub async fn find_type(&self, id: &str) -> StorageResult<Option<CardCodeType>> {
        self.find_type_record(id).await.map(|record| record.map(CardCodeType::from))
    }

    pub async fn list_types(
        &self,
        request: PageSliceRequest,
        filters: CardCodeTypeListFilters,
    ) -> StorageResult<Page<CardCodeType>> {
        let query = query::filtered_types(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(card_code_type_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(CardCodeType::from)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn code_exists(&self, code: &str) -> StorageResult<bool> {
        card_code_records::Entity::find()
            .filter(card_code_records::Column::Code.eq(code))
            .one(self.database.connection())
            .await
            .map(|record| record.is_some())
            .map_err(StorageError::from)
    }

    pub async fn create_codes(&self, inputs: Vec<CardCodeCreateRecord>) -> StorageResult<Vec<CardCode>> {
        if inputs.is_empty() {
            return Ok(Vec::new());
        }
        let records = inputs.into_iter().map(|input| self.code_active_model(input));
        let inserted = card_code_records::Entity::insert_many(records)
            .exec_with_returning(self.database.connection())
            .await?;
        Ok(inserted.into_iter().map(CardCode::from).collect())
    }

    pub async fn list_codes(
        &self,
        request: PageSliceRequest,
        filters: CardCodeListFilters,
    ) -> StorageResult<Page<CardCode>> {
        let query = query::filtered_codes(filters);
        let total = query.clone().count(self.database.connection()).await?;
        let items = query
            .order_by_desc(card_code_records::Column::CreatedAt)
            .limit(request.limit)
            .offset(request.offset)
            .all(self.database.connection())
            .await?
            .into_iter()
            .map(CardCode::from)
            .collect();
        Ok(page(items, total, request))
    }

    pub async fn batch_update_code_status(&self, ids: &[String], status: &str) -> StorageResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }
        let now = time::OffsetDateTime::now_utc();
        let result = card_code_records::Entity::update_many()
            .col_expr(card_code_records::Column::Status, Expr::value(status))
            .col_expr(card_code_records::Column::UpdatedAt, Expr::value(now))
            .filter(card_code_records::Column::Id.is_in(ids.iter().cloned()))
            .filter(card_code_records::Column::Status.is_in(active_or_disabled()))
            .filter(card_code_records::Column::UsedAt.is_null())
            .filter(query::not_expired_condition(now))
            .exec(self.database.connection())
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn redeem(&self, input: CardCodeRedeemInput) -> StorageResult<CardCodeRedeemRecord> {
        redemption::redeem(&self.database, input).await
    }

    async fn find_type_record(
        &self,
        id: &str,
    ) -> StorageResult<Option<crate::card_code::CardCodeTypeRecord>> {
        card_code_type_records::Entity::find_by_id(id.to_owned())
            .one(self.database.connection())
            .await
            .map_err(StorageError::from)
    }

    fn code_active_model(&self, input: CardCodeCreateRecord) -> card_code_records::ActiveModel {
        let now = time::OffsetDateTime::now_utc();
        card_code_records::ActiveModel {
            id: Set(self.database.next_id()),
            code: Set(input.code),
            batch_no: Set(input.batch_no),
            type_id: Set(input.type_id),
            type_name: Set(input.type_name),
            recharge_amount: Set(input.recharge_amount),
            gift_amount: Set(input.gift_amount),
            status: Set(input.status),
            remark: Set(input.remark),
            expires_at: Set(time_format::parse_optional(input.expires_at.as_deref())),
            created_by_user_id: Set(input.created_by_user_id),
            created_by_username: Set(input.created_by_username),
            created_ip: Set(input.created_ip),
            used_by_user_id: Set(None),
            used_by_username: Set(None),
            used_ip: Set(None),
            used_at: Set(None),
            wallet_id: Set(None),
            wallet_transaction_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }
    }
}

fn active_or_disabled() -> [&'static str; 2] {
    [CARD_CODE_STATUS_ACTIVE, CARD_CODE_STATUS_DISABLED]
}

fn page<T>(items: Vec<T>, total: u64, request: PageSliceRequest) -> Page<T> {
    Page {
        items,
        total,
        page: request.page,
        page_size: request.page_size,
    }
}
