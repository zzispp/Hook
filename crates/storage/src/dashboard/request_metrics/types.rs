use rust_decimal::Decimal;

use super::constants::{
    HISTOGRAM_BOUNDS_MS, METRIC_FIRST_OUTPUT, METRIC_FIRST_SSE_EVENT, METRIC_LATENCY, METRIC_RESPONSE_HEADERS, METRIC_SSE_TO_OUTPUT, METRIC_TTFB,
};

pub(super) struct MetricContribution {
    pub(super) source_type: String,
    pub(super) created_at: time::OffsetDateTime,
    pub(super) user_id: Option<String>,
    pub(super) username: Option<String>,
    pub(super) token_id: Option<String>,
    pub(super) token_name: Option<String>,
    pub(super) token_prefix: Option<String>,
    pub(super) provider_id: Option<String>,
    pub(super) provider_name: Option<String>,
    pub(super) global_model_id: Option<String>,
    pub(super) model_name: Option<String>,
    pub(super) client_api_format: Option<String>,
    pub(super) provider_api_format: Option<String>,
    pub(super) is_stream: Option<bool>,
    pub(super) needs_conversion: Option<bool>,
    pub(super) request_count: i64,
    pub(super) success_count: i64,
    pub(super) failed_count: i64,
    pub(super) active_count: i64,
    pub(super) prompt_tokens: i64,
    pub(super) completion_tokens: i64,
    pub(super) cache_creation_input_tokens: i64,
    pub(super) cache_read_input_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) output_tokens: i64,
    pub(super) total_cost: Decimal,
    pub(super) base_cost: Decimal,
    pub(super) upstream_total_cost: Decimal,
    pub(super) cache_read_cost: Decimal,
    pub(super) cache_creation_cost: Decimal,
    pub(super) latency_total_ms: i64,
    pub(super) latency_sample_count: i64,
    pub(super) ttfb_total_ms: i64,
    pub(super) ttfb_sample_count: i64,
    pub(super) response_headers_total_ms: i64,
    pub(super) response_headers_sample_count: i64,
    pub(super) first_sse_event_total_ms: i64,
    pub(super) first_sse_event_sample_count: i64,
    pub(super) first_output_total_ms: i64,
    pub(super) first_output_sample_count: i64,
    pub(super) sse_to_output_total_ms: i64,
    pub(super) sse_to_output_sample_count: i64,
    pub(super) tps_latency_total_ms: i64,
    pub(super) tps_output_tokens: i64,
    pub(super) tps_sample_count: i64,
    pub(super) retry_count: i64,
    pub(super) failover_count: i64,
    pub(super) timeout_count: i64,
    pub(super) rate_limited_count: i64,
    pub(super) server_error_count: i64,
    pub(super) quota_limited_count: i64,
    pub(super) slow_request_count: i64,
}

pub(super) struct HistogramContribution {
    pub(super) source_type: String,
    pub(super) created_at: time::OffsetDateTime,
    pub(super) provider_id: Option<String>,
    pub(super) provider_name: Option<String>,
    pub(super) global_model_id: Option<String>,
    pub(super) provider_api_format: Option<String>,
    pub(super) is_stream: Option<bool>,
    pub(super) needs_conversion: Option<bool>,
    pub(super) latency_ms: Option<i64>,
    pub(super) ttfb_ms: Option<i64>,
    pub(super) response_headers_ms: Option<i64>,
    pub(super) first_sse_event_ms: Option<i64>,
    pub(super) first_output_ms: Option<i64>,
    pub(super) sse_to_output_ms: Option<i64>,
}

pub(super) struct HistogramSample {
    pub(super) metric_kind: &'static str,
    pub(super) le_ms: i64,
}

impl HistogramContribution {
    pub(super) fn samples(&self) -> Vec<HistogramSample> {
        let mut output = Vec::new();
        append_samples(&mut output, METRIC_LATENCY, self.latency_ms);
        append_samples(&mut output, METRIC_TTFB, self.ttfb_ms);
        append_samples(&mut output, METRIC_RESPONSE_HEADERS, self.response_headers_ms);
        append_samples(&mut output, METRIC_FIRST_SSE_EVENT, self.first_sse_event_ms);
        append_samples(&mut output, METRIC_FIRST_OUTPUT, self.first_output_ms);
        append_samples(&mut output, METRIC_SSE_TO_OUTPUT, self.sse_to_output_ms);
        output
    }
}

fn append_samples(output: &mut Vec<HistogramSample>, metric_kind: &'static str, value: Option<i64>) {
    let Some(value) = value else {
        return;
    };
    let value = value.max(0);
    for bound in HISTOGRAM_BOUNDS_MS {
        if value <= bound {
            output.push(HistogramSample { metric_kind, le_ms: bound });
            return;
        }
    }
    output.push(HistogramSample {
        metric_kind,
        le_ms: *HISTOGRAM_BOUNDS_MS.last().expect("histogram bounds must not be empty"),
    });
}
