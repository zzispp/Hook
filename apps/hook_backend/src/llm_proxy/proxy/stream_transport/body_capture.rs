use std::collections::VecDeque;

use serde_json::Value;
use types::model::PatchField;

use crate::llm_proxy::proxy::response_payload::body_value;

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct StreamResponseBodyPatches {
    pub(super) provider_response_body: PatchField<Value>,
    pub(super) client_response_body: PatchField<Value>,
}

#[derive(Default)]
pub(super) struct StreamBodyCapture {
    provider: Vec<u8>,
    client_sent: Vec<u8>,
    provider_frame_count: usize,
    client_sent_frame_count: usize,
}

impl StreamBodyCapture {
    pub(super) fn record_provider(&mut self, bytes: &[u8]) {
        self.provider_frame_count += 1;
        self.provider.extend_from_slice(bytes);
    }

    pub(super) fn record_client_sent(&mut self, bytes: &[u8]) {
        self.client_sent_frame_count += 1;
        self.client_sent.extend_from_slice(bytes);
    }

    pub(super) fn provider_frame_count(&self) -> usize {
        self.provider_frame_count
    }

    pub(super) fn client_sent_frame_count(&self) -> usize {
        self.client_sent_frame_count
    }

    pub(super) fn terminal_bodies(&self, pending: &VecDeque<req::Bytes>) -> StreamResponseBodyPatches {
        StreamResponseBodyPatches {
            provider_response_body: body_patch(&self.provider),
            client_response_body: body_patch(&client_body_with_pending(&self.client_sent, pending)),
        }
    }

    pub(super) fn cancelled_bodies(&self) -> StreamResponseBodyPatches {
        StreamResponseBodyPatches {
            provider_response_body: body_patch(&self.provider),
            client_response_body: body_patch(&self.client_sent),
        }
    }
}

fn client_body_with_pending(sent: &[u8], pending: &VecDeque<req::Bytes>) -> Vec<u8> {
    let mut bytes = sent.to_vec();
    for item in pending {
        bytes.extend_from_slice(item);
    }
    bytes
}

fn body_patch(bytes: &[u8]) -> PatchField<Value> {
    if bytes.is_empty() {
        return PatchField::Null;
    }
    PatchField::Value(body_value(bytes))
}

#[cfg(test)]
mod tests {
    use super::StreamBodyCapture;
    use serde_json::Value;
    use std::collections::VecDeque;
    use types::model::PatchField;

    #[test]
    fn terminal_bodies_include_provider_and_pending_client_bytes() {
        let mut capture = StreamBodyCapture::default();
        let pending = VecDeque::from([req::Bytes::from_static(b"data: second\n\n")]);

        capture.record_provider(b"data: first\n\n");
        capture.record_provider(b"data: second\n\n");
        capture.record_client_sent(b"data: first\n\n");

        let bodies = capture.terminal_bodies(&pending);

        assert_eq!(
            bodies.provider_response_body,
            PatchField::Value(Value::String("data: first\n\ndata: second\n\n".into()))
        );
        assert_eq!(
            bodies.client_response_body,
            PatchField::Value(Value::String("data: first\n\ndata: second\n\n".into()))
        );
    }

    #[test]
    fn cancelled_bodies_keep_only_sent_client_bytes() {
        let mut capture = StreamBodyCapture::default();

        capture.record_provider(b"data: first\n\n");
        capture.record_provider(b"data: second\n\n");
        capture.record_client_sent(b"data: first\n\n");

        let bodies = capture.cancelled_bodies();

        assert_eq!(
            bodies.provider_response_body,
            PatchField::Value(Value::String("data: first\n\ndata: second\n\n".into()))
        );
        assert_eq!(bodies.client_response_body, PatchField::Value(Value::String("data: first\n\n".into())));
    }
}
