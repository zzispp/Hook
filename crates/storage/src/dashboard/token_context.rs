use crate::provider::record::request_records;

pub(super) fn sum_cache_creation_tokens_sql(alias: &str) -> String {
    format!("COALESCE(SUM({}), 0)::bigint", cache_creation_tokens_sql(alias))
}

pub(super) fn sum_cache_read_tokens_sql(alias: &str) -> String {
    format!("COALESCE(SUM({}), 0)::bigint", cache_read_tokens_sql(alias))
}

pub(super) fn sum_total_tokens_sql(alias: &str) -> String {
    format!("COALESCE(SUM({}), 0)::bigint", total_tokens_sql(alias))
}

pub(super) fn total_tokens(record: &request_records::Model) -> i64 {
    base_total_tokens(record) + cache_creation_tokens(record) + cache_read_tokens(record)
}

pub(super) fn cache_creation_tokens(record: &request_records::Model) -> i64 {
    let split_total = positive(record.cache_creation_5m_input_tokens) + positive(record.cache_creation_1h_input_tokens);
    let total = positive(record.cache_creation_input_tokens);
    if total == 0 && split_total > 0 {
        return split_total;
    }
    total
}

pub(super) fn cache_read_tokens(record: &request_records::Model) -> i64 {
    positive(record.cache_read_input_tokens)
}

fn total_tokens_sql(alias: &str) -> String {
    format!(
        "({}) + ({}) + ({})",
        base_total_tokens_sql(alias),
        cache_creation_tokens_sql(alias),
        cache_read_tokens_sql(alias)
    )
}

fn base_total_tokens_sql(alias: &str) -> String {
    format!("GREATEST(COALESCE({alias}.total_tokens, COALESCE({alias}.prompt_tokens, 0) + COALESCE({alias}.completion_tokens, 0), 0), 0)")
}

fn cache_creation_tokens_sql(alias: &str) -> String {
    format!(
        "CASE WHEN COALESCE({alias}.cache_creation_input_tokens, 0) = 0 \
        AND (COALESCE({alias}.cache_creation_5m_input_tokens, 0) + COALESCE({alias}.cache_creation_1h_input_tokens, 0)) > 0 \
        THEN GREATEST(COALESCE({alias}.cache_creation_5m_input_tokens, 0), 0) + GREATEST(COALESCE({alias}.cache_creation_1h_input_tokens, 0), 0) \
        ELSE GREATEST(COALESCE({alias}.cache_creation_input_tokens, 0), 0) END"
    )
}

fn cache_read_tokens_sql(alias: &str) -> String {
    format!("GREATEST(COALESCE({alias}.cache_read_input_tokens, 0), 0)")
}

fn base_total_tokens(record: &request_records::Model) -> i64 {
    record
        .total_tokens
        .unwrap_or_else(|| record.prompt_tokens.unwrap_or_default() + record.completion_tokens.unwrap_or_default())
        .max(0)
}

fn positive(value: Option<i64>) -> i64 {
    value.unwrap_or_default().max(0)
}

#[cfg(test)]
mod tests {
    use super::positive;

    #[test]
    fn positive_clamps_missing_and_negative_tokens_to_zero() {
        assert_eq!(positive(None), 0);
        assert_eq!(positive(Some(-7)), 0);
        assert_eq!(positive(Some(11)), 11);
    }
}
