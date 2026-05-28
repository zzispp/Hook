#[path = "entities/mod.rs"]
pub mod entities;

pub use entities::{payment_callback_records, payment_channels, recharge_orders, recharge_packages};

pub type RechargePackageRecord = recharge_packages::Model;
pub type RechargeOrderRecord = recharge_orders::Model;
pub type PaymentChannelRecord = payment_channels::Model;
pub type PaymentCallbackRecord = payment_callback_records::Model;
