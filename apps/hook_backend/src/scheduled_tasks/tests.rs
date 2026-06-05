use scheduler::runtime::ScheduledTaskLifecycle;

use super::RequestRecordStaleSweepTask;

#[test]
fn request_record_stale_sweep_definition_matches_runtime_contract() {
    let definition = RequestRecordStaleSweepTask.definition();

    assert_eq!(definition.code, "request_record_stale_sweep");
    assert_eq!(definition.name_key, "scheduledTasks.definitions.requestRecordStaleSweep.name");
    assert_eq!(definition.description_key, "scheduledTasks.definitions.requestRecordStaleSweep.description");
    assert_eq!(definition.default_interval_seconds, 300);
    assert_eq!(definition.default_config["pending_timeout_minutes"], 10);
    assert_eq!(definition.default_config["streaming_timeout_minutes"], 10);
    assert_eq!(definition.config_schema.len(), 2);
    assert_eq!(definition.config_schema[0].key, "pending_timeout_minutes");
    assert_eq!(definition.config_schema[0].min, Some(1));
    assert_eq!(definition.config_schema[1].key, "streaming_timeout_minutes");
    assert_eq!(definition.config_schema[1].min, Some(1));
}

#[test]
fn request_record_stale_sweep_config_requires_positive_timeouts() {
    let task = RequestRecordStaleSweepTask;

    assert!(
        task.validate_config(&serde_json::json!({
            "pending_timeout_minutes": 10,
            "streaming_timeout_minutes": 10
        }))
        .is_ok()
    );
    assert!(
        task.validate_config(&serde_json::json!({
            "pending_timeout_minutes": 0,
            "streaming_timeout_minutes": 10
        }))
        .is_err()
    );
    assert!(task.validate_config(&serde_json::json!({"pending_timeout_minutes": 10})).is_err());
}
