use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use types::recharge::PaymentChannel;

use super::{PaymentChannelDefinition, PaymentChannelRecord, PaymentChannelRecordPatch, RechargeStore, record::payment_channels as payment_channel_records};
use crate::{StorageError, StorageResult, json};

impl RechargeStore {
    pub async fn payment_channel_record(&self, code: &str) -> StorageResult<Option<PaymentChannelRecord>> {
        payment_channel_records::Entity::find_by_id(code.to_owned())
            .one(self.database.connection())
            .await
            .map_err(Into::into)
    }

    pub async fn update_payment_channel(&self, code: &str, patch: PaymentChannelRecordPatch) -> StorageResult<PaymentChannel> {
        let record = self.payment_channel_record(code).await?.ok_or(StorageError::NotFound)?;
        let mut active: payment_channel_records::ActiveModel = record.into();
        active.enabled = Set(patch.enabled);
        if let Some(config) = patch.config {
            active.config_json = Set(json::encode_required(&config)?);
        }
        if let Some(encrypted_secret) = patch.encrypted_secret {
            active.encrypted_secret = Set(encrypted_secret);
        }
        active.updated_at = Set(time::OffsetDateTime::now_utc());
        Ok(active.update(self.database.connection()).await?.into())
    }

    pub(super) async fn sync_payment_channel(&self, definition: &PaymentChannelDefinition) -> StorageResult<()> {
        let now = time::OffsetDateTime::now_utc();
        match self.payment_channel_record(&definition.code).await? {
            Some(record) => self.update_registered_channel(record, definition, now).await,
            None => self.insert_registered_channel(definition, now).await,
        }
    }

    async fn update_registered_channel(
        &self,
        record: PaymentChannelRecord,
        definition: &PaymentChannelDefinition,
        now: time::OffsetDateTime,
    ) -> StorageResult<()> {
        let mut active: payment_channel_records::ActiveModel = record.into();
        active.name = Set(definition.name.clone());
        active.updated_at = Set(now);
        active.update(self.database.connection()).await?;
        Ok(())
    }

    async fn insert_registered_channel(&self, definition: &PaymentChannelDefinition, now: time::OffsetDateTime) -> StorageResult<()> {
        payment_channel_records::ActiveModel {
            code: Set(definition.code.clone()),
            name: Set(definition.name.clone()),
            enabled: Set(false),
            config_json: Set("{}".into()),
            encrypted_secret: Set(String::new()),
            registered_at: Set(now),
            updated_at: Set(now),
        }
        .insert(self.database.connection())
        .await?;
        Ok(())
    }
}
