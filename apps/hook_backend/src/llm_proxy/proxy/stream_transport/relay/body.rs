use super::{DownstreamItem, StreamRelay};
use crate::llm_proxy::proxy::stream_transport::body_capture::StreamResponseBodyPatches;

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
        self.body_capture.terminal_bodies(&self.pending)
    }

    pub(super) fn cancelled_response_bodies(&self) -> StreamResponseBodyPatches {
        self.body_capture.cancelled_bodies()
    }
}
