use crate::provider::record::{request_candidates, request_records};

use super::{
    candidate,
    constants::STATUS_SUCCESS,
    request,
    types::{HistogramContribution, MetricContribution},
};

pub(super) struct RequestContribution {
    pub(super) metric: MetricContribution,
    pub(super) histogram: HistogramContribution,
}

impl RequestContribution {
    pub(super) fn from_record(record: &request_records::Model) -> Self {
        let success = record.status == STATUS_SUCCESS;
        Self {
            metric: request::metric(record),
            histogram: request::histogram(record, success),
        }
    }
}

pub(super) struct CandidateContribution {
    pub(super) metric: MetricContribution,
    pub(super) histogram: HistogramContribution,
}

impl CandidateContribution {
    pub(super) fn from_record(record: &request_candidates::Model) -> Option<Self> {
        if !candidate::is_started(record) {
            return None;
        }
        let success = candidate::is_success(record);
        Some(Self {
            metric: candidate::metric(record, success),
            histogram: candidate::histogram(record, success),
        })
    }
}
