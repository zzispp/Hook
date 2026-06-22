use super::{DownstreamItem, StreamRelay};
use crate::llm_proxy::proxy::{
    stream_transport::{body_capture::StreamResponseBodyPatches, terminal::StreamTerminalObservability},
    transport,
};

impl StreamRelay {
    pub(super) fn record_provider_body(&mut self, bytes: &[u8]) {
        self.body_capture.record_provider(bytes);
    }

    pub(super) fn client_item(&mut self, bytes: req::Bytes) -> DownstreamItem {
        self.body_capture.record_client_sent(&bytes);
        Ok(bytes)
    }

    pub(super) fn pop_client_item(&mut self) -> Option<DownstreamItem> {
        self.pending.pop_front().map(|bytes| self.client_item(bytes))
    }

    pub(super) fn terminal_response_bodies(&self) -> StreamResponseBodyPatches {
        if self.will_send_pending_terminal_bytes() {
            return self.body_capture.terminal_bodies(&self.pending);
        }
        self.body_capture.cancelled_bodies()
    }

    pub(super) fn cancelled_response_bodies(&self) -> StreamResponseBodyPatches {
        self.body_capture.cancelled_bodies()
    }

    pub(super) fn terminal_observability(&self) -> StreamTerminalObservability {
        StreamTerminalObservability {
            first_byte_time_ms: self.first_byte_time_ms,
            latency_ms: transport::elapsed_ms(self.context.started),
            bodies: self.terminal_response_bodies(),
            provider_frame_count: self.body_capture.provider_frame_count(),
            client_frame_count: self.terminal_client_frame_count(),
            received_response_count: self.stream_status.received_response_count(),
        }
    }

    pub(super) fn cancelled_observability(&self) -> StreamTerminalObservability {
        StreamTerminalObservability {
            first_byte_time_ms: self.first_byte_time_ms,
            latency_ms: transport::elapsed_ms(self.context.started),
            bodies: self.cancelled_response_bodies(),
            provider_frame_count: self.body_capture.provider_frame_count(),
            client_frame_count: self.body_capture.client_sent_frame_count(),
            received_response_count: self.stream_status.received_response_count(),
        }
    }

    fn terminal_client_frame_count(&self) -> usize {
        let sent = self.body_capture.client_sent_frame_count();
        if self.will_send_pending_terminal_bytes() {
            return sent + self.pending.len();
        }
        sent
    }

    fn will_send_pending_terminal_bytes(&self) -> bool {
        self.client_output_started || self.client_failure.is_none()
    }
}
