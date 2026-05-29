use sea_orm::{DbBackend, Statement, Value};
use types::model_status::ModelStatusListRequest;

const CHECK_ROWS_SELECT_SQL: &str = "SELECT c.id, c.name, c.global_model_id, COALESCE(NULLIF(g.display_name, ''), g.name) AS model_name, c.api_format, c.api_token_id, t.name AS api_token_name, c.interval_seconds, c.enabled, c.next_due_at, c.last_status, c.last_checked_at, c.last_latency_ms, c.last_message, c.created_at, c.updated_at FROM model_status_checks c JOIN global_models g ON g.id = c.global_model_id JOIN api_tokens t ON t.id = c.api_token_id";
const AVAILABILITY_SELECT_SQL: &str = "SELECT check_id, COALESCE(SUM(total_count), 0)::bigint AS total_checks, COALESCE(SUM(available_count), 0)::bigint AS available_checks FROM model_status_check_hourly_stats WHERE bucket_started_at >= $1 AND bucket_started_at < $2";

pub(super) fn check_rows_statement(request: &ModelStatusListRequest, public_only: bool) -> Statement {
    let mut query = CheckRowsQuery::new();
    query.add_public_filter(public_only);
    query.add_enabled_filter(request.enabled);
    query.add_api_format_filter(request.api_format.as_deref());
    query.add_search_filter(request.search.as_deref());
    query.statement()
}

pub(super) fn availability_statement(started_at: time::OffsetDateTime, ended_at: time::OffsetDateTime, check_ids: &[String]) -> Statement {
    let mut query = AvailabilityQuery::new(started_at, ended_at);
    query.add_check_ids(check_ids);
    query.statement()
}

struct CheckRowsQuery {
    where_parts: Vec<String>,
    values: Vec<Value>,
}

impl CheckRowsQuery {
    fn new() -> Self {
        Self {
            where_parts: Vec::new(),
            values: Vec::new(),
        }
    }

    fn add_public_filter(&mut self, public_only: bool) {
        if public_only {
            self.where_parts.push("c.enabled = TRUE".to_owned());
        }
    }

    fn add_enabled_filter(&mut self, enabled: Option<bool>) {
        if let Some(value) = enabled {
            let placeholder = self.push_value(value.into());
            self.where_parts.push(format!("c.enabled = {placeholder}"));
        }
    }

    fn add_api_format_filter(&mut self, api_format: Option<&str>) {
        let Some(value) = normalized(api_format) else {
            return;
        };
        let placeholder = self.push_value(value.into());
        self.where_parts.push(format!("c.api_format = {placeholder}"));
    }

    fn add_search_filter(&mut self, search: Option<&str>) {
        let Some(value) = normalized(search) else {
            return;
        };
        let placeholder = self.push_value(format!("%{value}%").into());
        self.where_parts.push(format!(
            "(g.name ILIKE {placeholder} OR g.display_name ILIKE {placeholder} OR c.api_format ILIKE {placeholder})"
        ));
    }

    fn statement(self) -> Statement {
        let mut sql = CHECK_ROWS_SELECT_SQL.to_owned();
        if !self.where_parts.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_parts.join(" AND "));
        }
        sql.push_str(" ORDER BY c.created_at DESC");
        Statement::from_sql_and_values(DbBackend::Postgres, sql, self.values)
    }

    fn push_value(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("${}", self.values.len())
    }
}

struct AvailabilityQuery {
    sql: String,
    values: Vec<Value>,
}

impl AvailabilityQuery {
    fn new(started_at: time::OffsetDateTime, ended_at: time::OffsetDateTime) -> Self {
        Self {
            sql: AVAILABILITY_SELECT_SQL.to_owned(),
            values: vec![started_at.into(), ended_at.into()],
        }
    }

    fn add_check_ids(&mut self, check_ids: &[String]) {
        let placeholders: Vec<String> = check_ids.iter().map(|check_id| self.push_value(check_id.to_owned().into())).collect();
        if !placeholders.is_empty() {
            self.sql.push_str(" AND check_id IN (");
            self.sql.push_str(&placeholders.join(", "));
            self.sql.push(')');
        }
    }

    fn statement(mut self) -> Statement {
        self.sql.push_str(" GROUP BY check_id");
        Statement::from_sql_and_values(DbBackend::Postgres, self.sql, self.values)
    }

    fn push_value(&mut self, value: Value) -> String {
        self.values.push(value);
        format!("${}", self.values.len())
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_rows_statement_parameterizes_filters() {
        let request = ModelStatusListRequest {
            search: Some("gpt%' OR 1=1 --".into()),
            api_format: Some("openai:chat".into()),
            enabled: Some(true),
            ..Default::default()
        };

        let statement = check_rows_statement(&request, true);

        assert!(statement.sql.contains("c.enabled = TRUE"));
        assert!(statement.sql.contains("c.enabled = $1"));
        assert!(statement.sql.contains("c.api_format = $2"));
        assert!(statement.sql.contains("g.name ILIKE $3"));
        assert!(statement.sql.contains("g.display_name ILIKE $3"));
        assert!(!statement.sql.contains("OR 1=1"));
        assert_eq!(statement.values.expect("filters must be bound").0.len(), 3);
    }

    #[test]
    fn availability_statement_reads_hourly_stats_for_selected_checks() {
        let started_at = time::OffsetDateTime::UNIX_EPOCH;
        let ended_at = started_at + time::Duration::hours(1);
        let check_ids = vec!["check-1".to_owned(), "check-2".to_owned()];

        let statement = availability_statement(started_at, ended_at, &check_ids);

        assert!(statement.sql.contains("model_status_check_hourly_stats"));
        assert!(!statement.sql.contains("model_status_check_runs"));
        assert!(statement.sql.contains("check_id IN ($3, $4)"));
        assert_eq!(statement.values.expect("availability filters must be bound").0.len(), 4);
    }
}
