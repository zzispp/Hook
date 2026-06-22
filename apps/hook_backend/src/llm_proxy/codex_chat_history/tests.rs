use serde_json::json;

use super::{test_support::test_history, test_support::test_history_with_ttl, types::CodexChatHistoryError, *};

#[tokio::test]
async fn enriches_tool_output_with_cached_function_call_from_previous_response() {
    let history = history_with_call("resp_1", "call_1").await;
    let mut request = json!({
        "previous_response_id": "resp_1",
        "input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]
    });

    let enrichment = history.enrich_request(&mut request).await.unwrap();

    assert_eq!(enrichment.restored_count, 1);
    assert_eq!(request["input"][0]["type"], "function_call");
    assert_eq!(request["input"][0]["reasoning_content"], "Need to inspect the file.");
    assert_eq!(request["input"][1]["type"], "function_call_output");
}

#[tokio::test]
async fn enriches_existing_function_call_missing_fields() {
    let history = history_with_call("resp_1", "call_1").await;
    let mut request = json!({
        "previous_response_id": "resp_1",
        "input": [
            {"type": "function_call", "call_id": "call_1"},
            {"type": "function_call_output", "call_id": "call_1", "output": "ok"}
        ]
    });

    let enrichment = history.enrich_request(&mut request).await.unwrap();

    assert_eq!(enrichment.enriched_count, 1);
    assert_eq!(request["input"][0]["name"], "read_file");
    assert_eq!(request["input"][0]["arguments"], "{\"path\":\"README.md\"}");
    assert_eq!(request["input"][0]["status"], "completed");
    assert_eq!(request["input"][0]["reasoning_content"], "Need to inspect the file.");
}

#[tokio::test]
async fn restores_parallel_tool_calls_as_one_group() {
    let history = test_history().await;
    history
        .record_response(&json!({
            "id": "resp_1",
            "output": [
                function_call("call_a"),
                function_call("call_b")
            ]
        }))
        .await
        .unwrap();
    let mut request = json!({
        "previous_response_id": "resp_1",
        "input": [
            {"type": "function_call_output", "call_id": "call_b", "output": "two"},
            {"type": "function_call_output", "call_id": "call_a", "output": "one"}
        ]
    });

    let enrichment = history.enrich_request(&mut request).await.unwrap();

    assert_eq!(enrichment.restored_count, 2);
    assert_eq!(request["input"][0]["call_id"], "call_a");
    assert_eq!(request["input"][1]["call_id"], "call_b");
    assert_eq!(request["input"][2]["type"], "function_call_output");
}

#[tokio::test]
async fn restores_unique_call_id_without_previous_response() {
    let history = history_with_call("resp_1", "call_1").await;
    let mut request = json!({
        "previous_response_id": "resp_missing",
        "input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]
    });

    let enrichment = history.enrich_request(&mut request).await.unwrap();

    assert_eq!(enrichment.restored_count, 1);
    assert_eq!(request["input"][0]["type"], "function_call");
}

#[tokio::test]
async fn complete_existing_function_call_does_not_require_history() {
    let history = test_history().await;
    let mut request = json!({
        "input": [
            function_call("call_1"),
            {"type": "function_call_output", "call_id": "call_1", "output": "ok"}
        ]
    });

    let enrichment = history.enrich_request(&mut request).await.unwrap();

    assert_eq!(enrichment.restored_count, 0);
    assert_eq!(enrichment.enriched_count, 0);
    assert_eq!(request["input"][0]["type"], "function_call");
}

#[tokio::test]
async fn ambiguous_call_id_returns_error_without_mutating_request() {
    let history = test_history().await;
    for response_id in ["resp_1", "resp_2"] {
        history
            .record_response(&json!({"id": response_id, "output": [function_call("call_1")]}))
            .await
            .unwrap();
    }
    let mut request = json!({"input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]});

    let error = history.enrich_request(&mut request).await.unwrap_err();

    assert_eq!(
        error,
        CodexChatHistoryError::Ambiguous {
            call_ids: vec!["call_1".to_owned()]
        }
    );
    assert_eq!(request["input"][0]["type"], "function_call_output");
}

#[tokio::test]
async fn missing_call_id_returns_error_without_mutating_request() {
    let history = test_history().await;
    let mut request = json!({
        "previous_response_id": "resp_missing",
        "input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]
    });

    let error = history.enrich_request(&mut request).await.unwrap_err();

    assert_eq!(
        error,
        CodexChatHistoryError::Missing {
            previous_response_id: Some("resp_missing".to_owned()),
            call_ids: vec!["call_1".to_owned()]
        }
    );
    assert_eq!(request["input"][0]["type"], "function_call_output");
}

#[tokio::test]
async fn expires_old_responses_and_call_index() {
    let history = test_history_with_ttl(1).await;
    for index in 0..2 {
        history
            .record_response(&json!({"id": format!("resp_{index}"), "output": [function_call(&format!("call_{index}"))]}))
            .await
            .unwrap();
    }
    let mut old_request = json!({"input": [{"type": "function_call_output", "call_id": "call_0", "output": "old"}]});
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    assert!(matches!(
        history.enrich_request(&mut old_request).await,
        Err(CodexChatHistoryError::Missing { .. })
    ));
}

#[tokio::test]
async fn records_response_completed_stream_event() {
    let history = test_history().await;
    let recorded = history
        .record_stream_event(&json!({
            "type": "response.completed",
            "response": {"id": "resp_1", "output": [function_call("call_1")]}
        }))
        .await
        .unwrap();
    let mut request = json!({
        "previous_response_id": "resp_1",
        "input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]
    });

    assert_eq!(recorded, 1);
    assert_eq!(history.enrich_request(&mut request).await.unwrap().restored_count, 1);
}

#[tokio::test]
async fn records_output_item_done_stream_event() {
    let history = test_history().await;
    let recorded = history
        .record_stream_event(&json!({
            "type": "response.output_item.done",
            "response": {"id": "resp_1"},
            "item": function_call("call_1")
        }))
        .await
        .unwrap();
    let mut request = json!({
        "previous_response_id": "resp_1",
        "input": [{"type": "function_call_output", "call_id": "call_1", "output": "ok"}]
    });

    assert_eq!(recorded, 1);
    assert_eq!(history.enrich_request(&mut request).await.unwrap().restored_count, 1);
}

async fn history_with_call(response_id: &str, call_id: &str) -> CodexChatHistoryStore {
    let history = test_history().await;
    history
        .record_response(&json!({
            "id": response_id,
            "output": [function_call(call_id)]
        }))
        .await
        .unwrap();
    history
}

fn function_call(call_id: &str) -> serde_json::Value {
    json!({
        "type": "function_call",
        "call_id": call_id,
        "name": "read_file",
        "arguments": "{\"path\":\"README.md\"}",
        "status": "completed",
        "reasoning": {"summary": []},
        "reasoning_content": "Need to inspect the file."
    })
}
