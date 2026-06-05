use rust_decimal::Decimal;
use sea_orm::Value;
use types::user::{AdminAffiliateCommissionQuery, AdminAffiliateRelationChangeQuery, AdminAffiliateRelationQuery, AdminAffiliateReportQuery};

#[derive(Clone)]
pub(super) struct AffiliateSqlFilter {
    parts: Vec<String>,
    values: Vec<Value>,
    active_rule: &'static str,
}

impl AffiliateSqlFilter {
    pub(super) fn relation(query: &AdminAffiliateRelationQuery) -> Self {
        let mut filter = Self::new("u.is_deleted = FALSE");
        filter.add_user_search("u", query.user_search.as_deref());
        filter.add_user_search("ref", query.referrer_search.as_deref());
        filter.add_has_referrer(query.has_referrer);
        filter.add_raw_range("u.referred_at", query.referred_start.as_deref(), query.referred_end.as_deref());
        filter.parts.push("u.id <> 'system'".into());
        filter
    }

    pub(super) fn relation_change(query: &AdminAffiliateRelationChangeQuery) -> Self {
        let mut filter = Self::new("u.id IS NOT NULL");
        filter.add_user_search("u", query.user_search.as_deref());
        filter.add_user_search("op", query.operator_search.as_deref());
        filter.add_raw_range("ch.created_at", query.start_at.as_deref(), query.end_at.as_deref());
        filter
    }

    pub(super) fn commission(query: &AdminAffiliateCommissionQuery) -> Self {
        let mut filter = Self::new("r.is_deleted = FALSE AND u.is_deleted = FALSE");
        filter.add_user_search("r", query.referrer_search.as_deref());
        filter.add_user_search("u", query.referred_search.as_deref());
        filter.add_exact("c.recharge_order_id", query.recharge_order_id.as_deref());
        filter.add_raw_range("c.created_at", query.start_at.as_deref(), query.end_at.as_deref());
        filter.add_decimal_min("c.commission_amount", query.min_commission_amount);
        filter.add_decimal_max("c.commission_amount", query.max_commission_amount);
        filter
    }

    pub(super) fn report(query: &AdminAffiliateReportQuery) -> Self {
        let mut filter = Self::new("r.is_deleted = FALSE AND u.is_deleted = FALSE AND c.status = 'success'");
        filter.add_user_search("r", query.referrer_search.as_deref());
        filter.add_user_search("u", query.referred_search.as_deref());
        filter.add_raw_range("c.created_at", query.start_date.as_deref(), query.end_date.as_deref());
        filter
    }

    fn new(active_rule: &'static str) -> Self {
        Self {
            parts: Vec::new(),
            values: Vec::new(),
            active_rule,
        }
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

    pub(super) fn where_sql(&self) -> String {
        if self.parts.is_empty() {
            return format!("WHERE {}", self.active_rule);
        }
        format!("WHERE {} AND {}", self.active_rule, self.parts.join(" AND "))
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

    fn add_has_referrer(&mut self, value: Option<bool>) {
        match value {
            Some(true) => self.parts.push("u.referred_by_user_id IS NOT NULL".into()),
            Some(false) => self.parts.push("u.referred_by_user_id IS NULL".into()),
            None => {}
        }
    }

    fn add_exact(&mut self, column: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("{column} = {placeholder}"));
    }

    fn add_raw_range(&mut self, column: &str, start: Option<&str>, end: Option<&str>) {
        self.add_start(column, start);
        self.add_end(column, end);
    }

    fn add_start(&mut self, column: &str, value: Option<&str>) {
        let Some(value) = normalized(value) else {
            return;
        };
        let placeholder = self.push(value.into());
        self.parts.push(format!("{column} >= {placeholder}::timestamptz"));
    }

    fn add_end(&mut self, column: &str, value: Option<&str>) {
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
