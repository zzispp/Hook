use super::{AttemptOutcome, Candidate, UpstreamAttempt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateAuditRecord {
    pub request_id: String,
    pub token_id: Option<String>,
    pub group_code: Option<String>,
    pub global_model_id: Option<String>,
    pub provider_id: Option<String>,
    pub endpoint_id: Option<String>,
    pub key_id: Option<String>,
    pub client_api_format: String,
    pub provider_api_format: Option<String>,
    pub needs_conversion: bool,
    pub is_stream: bool,
    pub candidate_index: i32,
    pub retry_index: i32,
    pub status: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub first_byte_time_ms: Option<i64>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub started: bool,
    pub finished: bool,
}

pub struct CandidateAuditRecorder;

impl CandidateAuditRecorder {
    pub fn available_records(input: CandidateAuditInput<'_>, candidates: &[Candidate]) -> Vec<CandidateAuditRecord> {
        candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| base_record(input, candidate, index as i32, "available"))
            .collect()
    }

    pub fn no_candidate_record(input: CandidateAuditInput<'_>, error_message: String) -> CandidateAuditRecord {
        CandidateAuditRecord {
            request_id: input.request_id.to_owned(),
            token_id: input.token_id.map(str::to_owned),
            group_code: input.group_code.map(str::to_owned),
            global_model_id: input.global_model_id.map(str::to_owned),
            provider_id: None,
            endpoint_id: None,
            key_id: None,
            client_api_format: input.client_api_format.to_owned(),
            provider_api_format: None,
            needs_conversion: false,
            is_stream: input.is_stream,
            candidate_index: 0,
            retry_index: 0,
            status: "failed".into(),
            status_code: None,
            latency_ms: None,
            first_byte_time_ms: None,
            error_type: Some("no_candidate".into()),
            error_message: Some(error_message),
            started: false,
            finished: true,
        }
    }

    pub fn attempt_records(input: CandidateAuditInput<'_>, attempts: &[UpstreamAttempt]) -> Vec<CandidateAuditRecord> {
        attempts
            .iter()
            .enumerate()
            .map(|(index, attempt)| attempt_record(input, attempt, index as i32))
            .collect()
    }
}

#[derive(Clone, Copy)]
pub struct CandidateAuditInput<'a> {
    pub request_id: &'a str,
    pub token_id: Option<&'a str>,
    pub group_code: Option<&'a str>,
    pub global_model_id: Option<&'a str>,
    pub client_api_format: &'a str,
    pub is_stream: bool,
}

fn attempt_record(input: CandidateAuditInput<'_>, attempt: &UpstreamAttempt, index: i32) -> CandidateAuditRecord {
    let mut record = base_record(input, &attempt.candidate, index, status_for_outcome(&attempt.outcome));
    record.started = true;
    record.finished = true;
    match &attempt.outcome {
        AttemptOutcome::Success => {
            record.status_code = Some(200);
            record.error_type = None;
            record.error_message = None;
        }
        AttemptOutcome::RetryableFailure(message) => set_error(&mut record, "retryable_upstream_error", message),
        AttemptOutcome::FatalFailure(message) => set_error(&mut record, "fatal_upstream_error", message),
    }
    record
}

fn base_record(input: CandidateAuditInput<'_>, candidate: &Candidate, index: i32, status: &str) -> CandidateAuditRecord {
    CandidateAuditRecord {
        request_id: input.request_id.to_owned(),
        token_id: input.token_id.map(str::to_owned),
        group_code: input.group_code.map(str::to_owned),
        global_model_id: input.global_model_id.map(str::to_owned).or_else(|| Some(candidate.global_model_id.clone())),
        provider_id: Some(candidate.provider_id.clone()),
        endpoint_id: Some(candidate.endpoint_id.clone()),
        key_id: Some(candidate.key_id.clone()),
        client_api_format: input.client_api_format.to_owned(),
        provider_api_format: Some(format!("{:?}", candidate.provider_api_format)),
        needs_conversion: candidate.needs_conversion,
        is_stream: input.is_stream,
        candidate_index: index,
        retry_index: 0,
        status: status.to_owned(),
        status_code: None,
        latency_ms: None,
        first_byte_time_ms: None,
        error_type: None,
        error_message: None,
        started: false,
        finished: false,
    }
}

fn status_for_outcome(outcome: &AttemptOutcome) -> &'static str {
    match outcome {
        AttemptOutcome::Success => "success",
        AttemptOutcome::RetryableFailure(_) | AttemptOutcome::FatalFailure(_) => "failed",
    }
}

fn set_error(record: &mut CandidateAuditRecord, error_type: &str, message: &str) {
    record.error_type = Some(error_type.to_owned());
    record.error_message = Some(message.to_owned());
}
