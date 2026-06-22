use super::sub2api_import_types::Sub2ApiKeyRecord;

pub(super) fn skip_inactive_group_tokens(records: Vec<Sub2ApiKeyRecord>) -> Vec<Sub2ApiKeyRecord> {
    records.into_iter().filter(|record| !references_inactive_group(record)).collect()
}

fn references_inactive_group(record: &Sub2ApiKeyRecord) -> bool {
    let Some(group) = record.group.as_ref() else {
        return false;
    };
    !group.is_active()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_records_whose_group_is_inactive() {
        let records = vec![
            record(1, Some(group("plus", "active"))),
            record(2, Some(group("cheap", "disabled"))),
            record(3, None),
        ];

        let filtered = skip_inactive_group_tokens(records);

        assert_eq!(filtered.into_iter().map(|record| record.id).collect::<Vec<_>>(), vec![1, 3]);
    }

    fn record(id: i64, group: Option<super::super::sub2api_import_types::Sub2ApiGroupRecord>) -> Sub2ApiKeyRecord {
        Sub2ApiKeyRecord {
            id,
            key: format!("sk-{id}"),
            name: format!("token-{id}"),
            status: "active".into(),
            quota: 0.0,
            quota_used: 0.0,
            expires_at: None,
            group,
        }
    }

    fn group(name: &str, status: &str) -> super::super::sub2api_import_types::Sub2ApiGroupRecord {
        super::super::sub2api_import_types::Sub2ApiGroupRecord {
            id: 1,
            name: name.into(),
            rate_multiplier: 1.0,
            status: status.into(),
        }
    }
}
