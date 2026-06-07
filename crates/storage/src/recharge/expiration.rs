use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::Expr};
use types::recharge::{RECHARGE_ORDER_STATUS_EXPIRED, RECHARGE_ORDER_STATUS_PENDING};

use crate::StorageResult;

use super::{RechargeStore, record::recharge_orders as recharge_order_records};

impl RechargeStore {
    pub async fn expire_pending_orders(&self, now: time::OffsetDateTime) -> StorageResult<u64> {
        recharge_order_records::Entity::update_many()
            .col_expr(recharge_order_records::Column::Status, Expr::value(RECHARGE_ORDER_STATUS_EXPIRED))
            .col_expr(recharge_order_records::Column::UpdatedAt, Expr::value(now))
            .filter(recharge_order_records::Column::Status.eq(RECHARGE_ORDER_STATUS_PENDING))
            .filter(recharge_order_records::Column::ExpiresAt.lte(now))
            .exec(self.database.connection())
            .await
            .map(|result| result.rows_affected)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    use crate::{Database, recharge::RechargeStore};

    #[tokio::test]
    async fn expire_pending_orders_marks_due_pending_orders_expired() {
        let connection = MockDatabase::new(DatabaseBackend::Postgres)
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 3,
            }])
            .into_connection();
        let store = RechargeStore::new(Database::new(connection.clone()));

        let expired = store.expire_pending_orders(now()).await.unwrap();

        assert_eq!(expired, 3);
        let sql = connection
            .into_transaction_log()
            .iter()
            .flat_map(|entry| entry.statements())
            .map(|statement| statement.sql.clone())
            .collect::<Vec<_>>();
        assert!(sql.iter().any(|item| item.contains("UPDATE \"recharge_orders\"")), "{sql:?}");
        assert!(sql.iter().any(|item| item.contains("SET \"status\"")), "{sql:?}");
        assert!(sql.iter().any(|item| item.contains("\"recharge_orders\".\"status\" = ")), "{sql:?}");
        assert!(sql.iter().any(|item| item.contains("\"expires_at\" <= ")), "{sql:?}");
    }

    fn now() -> time::OffsetDateTime {
        time::Date::from_calendar_date(2026, time::Month::June, 7)
            .unwrap()
            .with_hms(10, 0, 0)
            .unwrap()
            .assume_utc()
    }
}
