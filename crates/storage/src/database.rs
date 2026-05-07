use crate::{StorageResult, user::UserRecord};

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
}

pub async fn connect_database(database_url: &str) -> StorageResult<Database> {
    let db = toasty::Db::builder().models(toasty::models!(UserRecord)).connect(database_url).await?;
    db.push_schema().await?;
    Ok(Database::new(db))
}
