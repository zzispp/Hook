use proxy::format_conversion::ApiFormat;

use super::StreamOutputStartDetector;

#[test]
fn stream_preoutput_openai_responses_preamble_does_not_start_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(b"data: {\"type\":\"response.created\"}\n\ndata: {\"type\":\"response.in_progress\"}\n\n")
        .unwrap();

    assert!(!started);
}

#[test]
fn stream_preoutput_openai_responses_delta_starts_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(b"data: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n")
        .unwrap();

    assert!(started);
}

#[test]
fn stream_preoutput_openai_responses_failed_event_does_not_start_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(b"data: {\"type\":\"response.failed\",\"response\":{\"error\":{\"message\":\"bad\"}}}\n\n")
        .unwrap();

    assert!(!started);
}

#[test]
fn stream_preoutput_openai_responses_content_part_added_starts_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(
            b"data: {\"type\":\"response.content_part.added\",\"output_index\":0,\"content_index\":0,\"part\":{\"type\":\"output_text\",\"text\":\"hello\"}}\n\n",
        )
        .unwrap();

    assert!(started);
}

#[test]
fn stream_preoutput_openai_responses_output_item_done_reasoning_starts_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(
            b"data: {\"type\":\"response.output_item.done\",\"output_index\":0,\"item\":{\"type\":\"reasoning\",\"summary\":[{\"type\":\"summary_text\",\"text\":\"thinking\"}]}}\n\n",
        )
        .unwrap();

    assert!(started);
}

#[test]
fn stream_preoutput_openai_responses_function_call_arguments_done_starts_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(
            b"data: {\"type\":\"response.function_call_arguments.done\",\"output_index\":0,\"item\":{\"type\":\"function_call\",\"name\":\"get_weather\",\"arguments\":\"{\\\"city\\\":\\\"Tokyo\\\"}\"}}\n\n",
        )
        .unwrap();

    assert!(started);
}

#[test]
fn stream_preoutput_openai_responses_message_refusal_starts_output() {
    let mut detector = StreamOutputStartDetector::new(ApiFormat::OpenAiResponses);

    let started = detector
        .consume(
            b"data: {\"type\":\"response.content_part.done\",\"output_index\":0,\"content_index\":0,\"part\":{\"type\":\"refusal\",\"refusal\":\"blocked\"}}\n\n",
        )
        .unwrap();

    assert!(started);
}
