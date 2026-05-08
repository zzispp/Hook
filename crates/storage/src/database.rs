use sea_orm::{ConnectOptions, DatabaseConnection};
use uuid::Uuid;

use crate::StorageResult;

#[derive(Clone)]
pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub fn new(connection: DatabaseConnection) -> Self {
        Self { connection }
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub(crate) fn next_id(&self) -> String {
        Uuid::now_v7().to_string()
    }

    pub fn into_inner(self) -> DatabaseConnection {
        self.connection
    }
}

pub async fn connect_database(database_url: &str) -> StorageResult<Database> {
    let mut options = ConnectOptions::new(database_url.to_owned());
    options.sqlx_logging(false);

    let connection = sea_orm::Database::connect(options).await?;
    Ok(Database::new(connection))
}
