use std::collections::BTreeMap;

use sea_orm::{DatabaseBackend, MockDatabase, Value};
use storage::{
    Database,
    user::{UserRecord, UserStore},
};
use types::{pagination::PageRequest, user::UserListFilters};

#[tokio::test]
async fn user_list_orders_by_created_at_desc() {
    let connection = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([[count_row(0)]])
        .append_query_results([Vec::<UserRecord>::new()])
        .into_connection();
    let store = UserStore::new(Database::new(connection.clone()));

    store.list(page_request(), UserListFilters::default()).await.unwrap();

    let logs = connection.into_transaction_log();
    let list_sql = &logs[1].statements()[0].sql;
    assert!(list_sql.contains(r#"ORDER BY "users"."created_at" DESC"#), "{list_sql}");
    assert!(list_sql.contains(r#""users"."id" ASC"#), "{list_sql}");
}

fn page_request() -> PageRequest {
    PageRequest { page: 1, page_size: 10 }
}

fn count_row(total: i64) -> BTreeMap<&'static str, Value> {
    BTreeMap::from([("num_items", total.into())])
}
