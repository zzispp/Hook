use crate::{
    StorageResult,
    rbac::{ApiPermissionRecord, MenuItemRecord, MenuSectionRecord, RoleApiPermissionRecord, RoleMenuPermissionRecord, RoleRecord},
    user::UserRecord,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    db: toasty::Db,
}

impl Database {
    pub fn new(db: toasty::Db) -> Self {
        Self { db }
    }

    pub(crate) fn connection(&self) -> toasty::Db {
        self.db.clone()
    }

    pub(crate) fn next_id(&self) -> String {
        Uuid::now_v7().to_string()
    }

    pub async fn push_schema(&self) -> StorageResult<()> {
        self.db.push_schema().await?;
        Ok(())
    }

    pub fn table_names(&self) -> Vec<String> {
        self.db.schema().db.tables.iter().map(|table| table.name.clone()).collect()
    }

    pub fn into_inner(self) -> toasty::Db {
        self.db
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatabaseConnectOptions {
    pub push_schema: bool,
}

pub async fn connect_database(database_url: &str, options: DatabaseConnectOptions) -> StorageResult<Database> {
    let db = toasty::Db::builder()
        .models(toasty::models!(
            UserRecord,
            RoleRecord,
            ApiPermissionRecord,
            MenuSectionRecord,
            MenuItemRecord,
            RoleApiPermissionRecord,
            RoleMenuPermissionRecord
        ))
        .connect(database_url)
        .await?;
    if options.push_schema {
        db.push_schema().await?;
    }
    Ok(Database::new(db))
}
