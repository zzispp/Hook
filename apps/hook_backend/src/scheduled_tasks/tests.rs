use scheduler::runtime::ScheduledTaskLifecycle;

use super::{RequestRecordCleanupTask, RequestRecordStaleSweepTask};

#[test]
fn request_record_cleanup_definition_matches_runtime_contract() {
    let definition = RequestRecordCleanupTask.definition();

    assert_eq!(definition.code, "request_record_cleanup");
    assert_eq!(definition.default_interval_seconds, 86_400);
    assert_eq!(definition.default_config["delete_batch_size"], 200);
    assert_eq!(definition.default_config["compress_batch_size"], 50);
    assert_eq!(definition.default_config["max_runtime_seconds"], 120);
    assert_eq!(definition.default_config["batch_sleep_ms"], 100);
    assert_eq!(definition.default_config["statement_timeout_seconds"], 15);
    assert_eq!(definition.default_config["lock_timeout_seconds"], 2);
    assert_eq!(definition.config_schema.len(), 8);
    assert_field(&definition, "delete_batch_size", 1);
    assert_field(&definition, "compress_batch_size", 1);
    assert_field(&definition, "max_runtime_seconds", 1);
    assert_field(&definition, "batch_sleep_ms", 0);
    assert_field(&definition, "statement_timeout_seconds", 1);
    assert_field(&definition, "lock_timeout_seconds", 1);
}

#[test]
fn request_record_cleanup_config_requires_new_batch_fields() {
    let task = RequestRecordCleanupTask;

    assert!(task.validate_config(&valid_cleanup_config()).is_ok());
    assert!(task.validate_config(&without_cleanup_field("delete_batch_size")).is_err());
    assert!(task.validate_config(&with_cleanup_field("compress_batch_size", 0)).is_err());
    assert!(task.validate_config(&with_cleanup_field("batch_sleep_ms", -1)).is_err());
}

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

fn assert_field(definition: &types::scheduler::ScheduledTaskDefinition, key: &str, min: i64) {
    let field = definition.config_schema.iter().find(|field| field.key == key).unwrap();
    assert_eq!(field.min, Some(min));
    assert!(field.required);
}

fn valid_cleanup_config() -> serde_json::Value {
    serde_json::json!({
        "record_retention_days": 3,
        "payload_retention_days": 1,
        "delete_batch_size": 200,
        "compress_batch_size": 50,
        "max_runtime_seconds": 120,
        "batch_sleep_ms": 100,
        "statement_timeout_seconds": 15,
        "lock_timeout_seconds": 2
    })
}

fn without_cleanup_field(key: &str) -> serde_json::Value {
    let mut value = valid_cleanup_config();
    value.as_object_mut().unwrap().remove(key);
    value
}

fn with_cleanup_field(key: &str, next: i64) -> serde_json::Value {
    let mut value = valid_cleanup_config();
    value.as_object_mut().unwrap().insert(key.into(), serde_json::json!(next));
    value
}
