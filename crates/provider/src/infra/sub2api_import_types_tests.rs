use super::sub2api_import_types::{Sub2ApiGroupRecord, Sub2ApiKeyRecord, UserGroupRates, group_ratio};

const ACTIVE_STATUS: &str = "active";

#[test]
fn group_ratio_rejects_inactive_group() {
    let record = key_record(group("cheap", "disabled"));

    let error = group_ratio(&record, &UserGroupRates::new()).unwrap_err();

    assert_eq!(error.to_string(), "infrastructure error: sub2api group is inactive: cheap");
}

fn key_record(group: Sub2ApiGroupRecord) -> Sub2ApiKeyRecord {
    Sub2ApiKeyRecord {
        id: 1,
        key: "sk-test".into(),
        name: "token".into(),
        status: ACTIVE_STATUS.into(),
        quota: 0.0,
        quota_used: 0.0,
        expires_at: None,
        group: Some(group),
    }
}

fn group(name: &str, status: &str) -> Sub2ApiGroupRecord {
    Sub2ApiGroupRecord {
        id: 1,
        name: name.into(),
        rate_multiplier: 1.0,
        status: status.into(),
    }
}
