use super::{BaselineApplyAction, BaselineState, baseline_apply_action, classify_baseline_state};

#[test]
fn classifies_empty_baseline_without_marker_as_empty() {
    assert_eq!(classify_baseline_state(0, 5, false, &[]), BaselineState::Empty);
}

#[test]
fn classifies_complete_baseline_without_marker_as_marker_pending() {
    assert_eq!(classify_baseline_state(5, 5, false, &[]), BaselineState::CompleteWithoutMarker);
}

#[test]
fn complete_baseline_without_marker_applies_additives_after_marker() {
    let state = classify_baseline_state(5, 5, false, &[]);

    assert_eq!(baseline_apply_action(state), BaselineApplyAction::MarkBaselineAndApplyAdditives);
}

#[test]
fn classifies_complete_marked_baseline_as_applied() {
    assert_eq!(classify_baseline_state(5, 5, true, &[]), BaselineState::Applied);
}

#[test]
fn classifies_marked_baseline_missing_only_additive_tables_as_applied() {
    assert_eq!(
        classify_baseline_state(5, 7, true, &["provider_groups", "billing_group_provider_groups"]),
        BaselineState::Applied
    );
}

#[test]
fn classifies_marked_baseline_missing_quick_import_sync_tables_as_applied() {
    assert_eq!(
        classify_baseline_state(
            63,
            67,
            true,
            &[
                "provider_quick_import_sources",
                "provider_quick_import_keys",
                "provider_quick_import_key_models",
                "provider_quick_import_sync_events",
            ],
        ),
        BaselineState::Applied
    );
}

#[test]
fn classifies_partial_baseline_as_inconsistent() {
    assert_eq!(
        classify_baseline_state(3, 5, false, &["providers", "api_tokens"]),
        BaselineState::Inconsistent {
            existing_tables: 3,
            total_tables: 5,
        }
    );
}

#[test]
fn classifies_marker_without_tables_as_inconsistent() {
    assert_eq!(
        classify_baseline_state(0, 5, true, &["providers", "api_tokens"]),
        BaselineState::Inconsistent {
            existing_tables: 0,
            total_tables: 5,
        }
    );
}
