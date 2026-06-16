use super::newapi_import_types::{GroupMap, NewApiTokenRecord};

pub(super) fn skip_unimportable_group_tokens(records: Vec<NewApiTokenRecord>, groups: &GroupMap) -> Vec<NewApiTokenRecord> {
    records.into_iter().filter(|record| !references_unimportable_group(record, groups)).collect()
}

fn references_unimportable_group(record: &NewApiTokenRecord, groups: &GroupMap) -> bool {
    let Some(group) = record.group.as_deref() else {
        return false;
    };
    let Some(upstream_group) = groups.get(group) else {
        return true;
    };
    !upstream_group.has_fixed_ratio()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::newapi_import_types::{GroupsEnvelope, decode_envelope};

    #[test]
    fn skips_records_whose_group_was_deleted_upstream() {
        let groups = groups(r#"{"data":{"plus":{"ratio":2}},"success":true}"#);
        let records = vec![record(1, Some("plus")), record(2, Some("kiro-power")), record(3, None)];

        let filtered = skip_unimportable_group_tokens(records, &groups);

        assert_eq!(filtered.into_iter().map(|record| record.id).collect::<Vec<_>>(), vec![1, 3]);
    }

    #[test]
    fn skips_records_whose_group_has_non_fixed_ratio() {
        let groups = groups(r#"{"data":{"plus":{"ratio":2},"auto":{"ratio":"自动"}},"success":true}"#);
        let records = vec![record(1, Some("plus")), record(2, Some("auto")), record(3, None)];

        let filtered = skip_unimportable_group_tokens(records, &groups);

        assert_eq!(filtered.into_iter().map(|record| record.id).collect::<Vec<_>>(), vec![1, 3]);
    }

    fn groups(payload: &str) -> GroupMap {
        decode_envelope::<GroupsEnvelope>(payload).unwrap().data
    }

    fn record(id: i64, group: Option<&str>) -> NewApiTokenRecord {
        NewApiTokenRecord {
            id,
            key: format!("masked-{id}"),
            status: 1,
            name: format!("token-{id}"),
            group: group.map(str::to_owned),
        }
    }
}
