use time::format_description::well_known::Rfc3339;

use crate::application::{PaymentChannelRegistry, RechargeService, RechargeUseCase};

use super::{support::MemoryRechargeRepository, support_fixtures::order};

#[tokio::test]
async fn expire_pending_orders_marks_only_due_pending_orders_expired() {
    let repository = MemoryRechargeRepository::default();
    repository.insert_order(expiring_order("due-pending", "R1001", "pending", -1));
    repository.insert_order(expiring_order("future-pending", "R1002", "pending", 30));
    repository.insert_order(expiring_order("due-paid", "R1003", "paid", -1));
    let service = RechargeService::new(repository.clone(), PaymentChannelRegistry::empty()).await.unwrap();

    let expired = service.expire_pending_orders().await.unwrap();

    assert_eq!(expired, 1);
    let orders = repository.orders();
    assert_eq!(orders[0].status, "expired");
    assert_eq!(orders[1].status, "pending");
    assert_eq!(orders[2].status, "paid");
}

fn expiring_order(id: &str, order_no: &str, status: &str, offset_minutes: i64) -> types::recharge::RechargeOrder {
    let mut item = order(id, order_no, status);
    item.expires_at = timestamp(offset_minutes);
    item
}

fn timestamp(offset_minutes: i64) -> String {
    (time::OffsetDateTime::now_utc() + time::Duration::minutes(offset_minutes))
        .format(&Rfc3339)
        .expect("test timestamp must format")
}
