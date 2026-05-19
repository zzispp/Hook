use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::{StorageError, StorageResult};

use super::{
    RegistrationEmailVerificationRecord, RegistrationEmailVerificationRecordInput, UserStore, registration_email_verifications,
    registration_email_verifications::ActiveModel as RegistrationEmailVerificationActiveModel,
    tokens::{registration_email_verification_active_model, registration_email_verification_record},
};

impl UserStore {
    pub async fn create_registration_email_verification(
        &self,
        input: RegistrationEmailVerificationRecordInput,
    ) -> StorageResult<RegistrationEmailVerificationRecord> {
        RegistrationEmailVerificationActiveModel {
            id: Set(self.database.next_id()),
            email: Set(input.email),
            code_hash: Set(input.code_hash),
            expires_at: Set(input.expires_at),
            consumed_at: Set(None),
            created_at: Set(time::OffsetDateTime::now_utc()),
        }
        .insert(self.database.connection())
        .await
        .map(registration_email_verification_record)
        .map_err(StorageError::from)
    }

    pub async fn consume_registration_email_verification(&self, email: &str, code_hash: &str, now: time::OffsetDateTime) -> StorageResult<bool> {
        let tx = self.database.connection().begin().await?;
        let Some(record) = find_registration_email_verification_in_tx(email, code_hash, &tx).await? else {
            tx.commit().await?;
            return Ok(false);
        };
        if record.consumed_at.is_some() || record.expires_at <= now {
            tx.commit().await?;
            return Ok(false);
        }
        let mut active = registration_email_verification_active_model(record);
        active.consumed_at = Set(Some(now));
        active.update(&tx).await?;
        tx.commit().await?;
        Ok(true)
    }
}

async fn find_registration_email_verification_in_tx(
    email: &str,
    code_hash: &str,
    tx: &sea_orm::DatabaseTransaction,
) -> StorageResult<Option<RegistrationEmailVerificationRecord>> {
    registration_email_verifications::Entity::find()
        .filter(registration_email_verifications::Column::Email.eq(email))
        .filter(registration_email_verifications::Column::CodeHash.eq(code_hash))
        .order_by_desc(registration_email_verifications::Column::CreatedAt)
        .one(tx)
        .await
        .map(|record| record.map(registration_email_verification_record))
        .map_err(StorageError::from)
}
