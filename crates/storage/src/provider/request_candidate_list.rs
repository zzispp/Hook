use sea_orm::{DbBackend, FromQueryResult, Statement, Value};
use types::provider::RequestCandidateListRequest;

use crate::StorageResult;

use super::{record::request_candidates, repository::ProviderStore, request_record_partition_columns::REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED};

pub async fn list_request_candidates(store: &ProviderStore, request: RequestCandidateListRequest) -> StorageResult<Vec<types::provider::RequestCandidate>> {
    let (where_clause, values) = candidate_list_where(request.request_id);
    let sql = format!(
        "SELECT r.* FROM ({}) r {where_clause} ORDER BY r.candidate_index ASC, r.retry_index ASC LIMIT {} OFFSET {}",
        candidate_summary_union_sql(),
        request.limit,
        request.skip
    );
    let records = request_candidates::Model::find_by_statement(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
        .all(store.connection())
        .await?;
    records.into_iter().map(|record| record.response()).collect()
}

fn candidate_list_where(request_id: Option<String>) -> (String, Vec<Value>) {
    request_id
        .map(|id| ("WHERE r.request_id = $1".to_owned(), vec![Value::from(id)]))
        .unwrap_or_else(|| (String::new(), Vec::new()))
}

fn candidate_summary_union_sql() -> String {
    format!(
        "SELECT {} FROM request_candidates_partitioned r UNION ALL SELECT {} FROM request_candidates r WHERE NOT EXISTS \
         (SELECT 1 FROM request_candidates_partitioned p WHERE p.id = r.id)",
        REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED, REQUEST_CANDIDATE_MODEL_COLUMNS_PARTITIONED
    )
}
