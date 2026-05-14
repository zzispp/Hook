use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{Database, StorageResult};

use super::{NotificationStateColumn, NotificationStateEntity, NotificationStateRecord};

#[derive(Clone)]
pub struct OperationsStore {
    pub(super) database: Database,
}

impl OperationsStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub(super) fn connection(&self) -> &DatabaseConnection {
        self.database.connection()
    }

    pub(super) fn next_id(&self) -> String {
        self.database.next_id()
    }

    pub(super) async fn notification_state(&self, user_id: &str, source_type: &str, source_id: &str) -> StorageResult<Option<NotificationStateRecord>> {
        NotificationStateEntity::find()
            .filter(NotificationStateColumn::UserId.eq(user_id))
            .filter(NotificationStateColumn::SourceType.eq(source_type))
            .filter(NotificationStateColumn::SourceId.eq(source_id))
            .one(self.connection())
            .await
            .map_err(Into::into)
    }
}
