pub(super) struct FirstSseEventDetector {
    buffer: Vec<u8>,
}

impl FirstSseEventDetector {
    pub(super) fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub(super) fn consume(&mut self, bytes: &[u8]) -> bool {
        self.buffer.extend_from_slice(bytes);
        while let Some(line) = self.next_line() {
            if is_effective_sse_data_line(&line) {
                return true;
            }
        }
        false
    }

    fn next_line(&mut self) -> Option<Vec<u8>> {
        let position = self.buffer.iter().position(|byte| *byte == b'\n')?;
        Some(self.buffer.drain(..=position).collect())
    }
}

fn is_effective_sse_data_line(line: &[u8]) -> bool {
    let Ok(line) = std::str::from_utf8(line) else {
        return false;
    };
    let Some(payload) = line.trim_end_matches(['\r', '\n']).strip_prefix("data:") else {
        return false;
    };
    let payload = payload.trim();
    !payload.is_empty() && payload != "[DONE]"
}

#[cfg(test)]
mod tests {
    use super::FirstSseEventDetector;

    #[test]
    fn ignores_keepalive_and_non_data_lines() {
        let mut detector = FirstSseEventDetector::new();

        let detected = detector.consume(b": keepalive\n\nevent: ping\n\n\n");

        assert!(!detected);
    }

    #[test]
    fn ignores_empty_data_and_done() {
        let mut detector = FirstSseEventDetector::new();

        assert!(!detector.consume(b"data:\n\n"));
        assert!(!detector.consume(b"data:   \n\n"));
        assert!(!detector.consume(b"data: [DONE]\n\n"));
    }

    #[test]
    fn detects_first_effective_data_event() {
        let mut detector = FirstSseEventDetector::new();

        let detected = detector.consume(b": keepalive\n\ndata: {\"type\":\"response.created\"}\n\n");

        assert!(detected);
    }

    #[test]
    fn detects_split_data_line_across_chunks() {
        let mut detector = FirstSseEventDetector::new();

        assert!(!detector.consume(b"data: {\"type\""));
        assert!(detector.consume(b":\"response.created\"}\n\n"));
    }
}
