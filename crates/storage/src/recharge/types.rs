use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePackageRecordInput {
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargePackageRecordPatch {
    pub name: String,
    pub description: Option<String>,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub status: String,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RechargeOrderRecordInput {
    pub user_id: String,
    pub package_id: Option<String>,
    pub package_name: String,
    pub recharge_amount: Decimal,
    pub gift_amount: Decimal,
    pub total_arrival_amount: Decimal,
    pub payable_amount: Decimal,
    pub status: String,
    pub payment_channel_code: Option<String>,
    pub payment_channel_name: Option<String>,
    pub expires_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentChannelDefinition {
    pub code: String,
    pub name: String,
}
