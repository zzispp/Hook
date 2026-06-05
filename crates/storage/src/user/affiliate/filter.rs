use rust_decimal::Decimal;
use sea_orm::Value;
use types::user::{AffiliateCommissionQuery, AffiliateReferralQuery};

pub(super) struct AffiliateSqlFilter {
    parts: Vec<String>,
    values: Vec<Value>,
    owner_placeholder: String,
}

impl AffiliateSqlFilter {
    pub(super) fn referrals(referrer_user_id: &str, query: &AffiliateReferralQuery) -> Self {
        let mut filter = Self::new(referrer_user_id);
        filter.add_user_search("u", query.search.as_deref());
        filter.add_time_start("u.referred_at", query.referred_start.as_deref());
        filter.add_time_end("u.referred_at", query.referred_end.as_deref());
        filter
    }

    pub(super) fn commissions(referrer_user_id: &str, query: &AffiliateCommissionQuery) -> Self {
        let mut filter = Self::new(referrer_user_id);
        filter.add_user_search("u", query.referred_search.as_deref());
        filter.add_exact("o.order_no", query.recharge_order_no.as_deref());
        filter.add_time_start("c.created_at", query.start_at.as_deref());
        filter.add_time_end("c.created_at", query.end_at.as_deref());
        filter.add_decimal_min("c.commission_amount", query.min_commission_amount);
        filter.add_decimal_max("c.commission_amount", query.max_commission_amount);
        filter
    }

    pub(super) fn owner_placeholder(&self) -> &str {
        &self.owner_placeholder
    }

    pub(super) fn push(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("${}", self.values.len())
    }

    pub(super) fn values(&self) -> Vec<Value> {
        self.values.clone()
    }

    pub(super) fn into_values(self) -> Vec<Value> {
        self.values
    }

    pub(super) fn where_sql(&self, owner_rule: String) -> String {
        let mut parts = vec![owner_rule];
        parts.extend(self.parts.iter().cloned());
        format!("WHERE {}", parts.join(" AND "))
    }

    fn new(referrer_user_id: &str) -> Self {
        let mut filter = Self {
            parts: Vec::new(),
            values: Vec::new(),
            owner_placeholder: String::new(),
        };
        filter.owner_placeholder = filter.push(referrer_user_id.to_owned().into());
        filter
    }

    fn add_user_search(&mut self, alias: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(format!("%{value}%").into());
        self.parts.push(format!(
            "({alias}.id ILIKE {placeholder} OR {alias}.username ILIKE {placeholder} OR {alias}.email ILIKE {placeholder} OR {alias}.affiliate_code ILIKE {placeholder})"
        ));
    }

    fn add_exact(&mut self, column: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("{column} = {placeholder}"));
    }

    fn add_time_start(&mut self, column: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("{column} >= {placeholder}::timestamptz"));
    }

    fn add_time_end(&mut self, column: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("{column} < {placeholder}::timestamptz"));
    }

    fn add_decimal_min(&mut self, column: &str, value: Option<Decimal>) {
        if let Some(value) = value {
            let placeholder = self.push(value.into());
            self.parts.push(format!("{column} >= {placeholder}"));
        }
    }

    fn add_decimal_max(&mut self, column: &str, value: Option<Decimal>) {
        if let Some(value) = value {
            let placeholder = self.push(value.into());
            self.parts.push(format!("{column} <= {placeholder}"));
        }
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}
