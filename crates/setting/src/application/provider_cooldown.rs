use types::provider::ProviderCooldownPolicy;

use super::{SettingError, SettingResult};

const MIN_STATUS_CODE: i32 = 100;
const MAX_STATUS_CODE: i32 = 599;

pub(super) fn validate_provider_cooldown_policy(policy: Option<&ProviderCooldownPolicy>) -> SettingResult<()> {
    let Some(policy) = policy else {
        return Ok(());
    };
    if policy.rules.is_empty() {
        return Ok(());
    }
    validate_positive_value("provider_cooldown_policy.window_seconds", policy.window_seconds)?;
    for (index, rule) in policy.rules.iter().enumerate() {
        validate_status_code_range(rule.status_code_start, rule.status_code_end)?;
        validate_positive_value("provider_cooldown_policy.failure_count", rule.failure_count)?;
        validate_positive_value("provider_cooldown_policy.cooldown_seconds", rule.cooldown_seconds)?;
        validate_no_overlapping_range(policy, index)?;
    }
    Ok(())
}

fn validate_positive_value(field: &str, value: i64) -> SettingResult<()> {
    if value <= 0 {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn validate_status_code_range(start: i32, end: i32) -> SettingResult<()> {
    if !(MIN_STATUS_CODE..=MAX_STATUS_CODE).contains(&start) || !(MIN_STATUS_CODE..=MAX_STATUS_CODE).contains(&end) {
        return Err(SettingError::InvalidInput(format!(
            "provider_cooldown_policy status code range must be between {MIN_STATUS_CODE} and {MAX_STATUS_CODE}"
        )));
    }
    if start > end {
        return Err(SettingError::InvalidInput(
            "provider_cooldown_policy.status_code_start must be less than or equal to status_code_end".into(),
        ));
    }
    Ok(())
}

fn validate_no_overlapping_range(policy: &ProviderCooldownPolicy, index: usize) -> SettingResult<()> {
    let current = &policy.rules[index];
    for previous in &policy.rules[..index] {
        if ranges_overlap(
            current.status_code_start,
            current.status_code_end,
            previous.status_code_start,
            previous.status_code_end,
        ) {
            return Err(SettingError::InvalidInput(format!(
                "provider_cooldown_policy contains overlapping status code ranges: {}-{} and {}-{}",
                previous.status_code_start, previous.status_code_end, current.status_code_start, current.status_code_end
            )));
        }
    }
    Ok(())
}

fn ranges_overlap(left_start: i32, left_end: i32, right_start: i32, right_end: i32) -> bool {
    left_start <= right_end && right_start <= left_end
}

#[cfg(test)]
mod tests {
    use types::provider::{ProviderCooldownPolicy, ProviderCooldownRule};

    use super::validate_provider_cooldown_policy;

    #[test]
    fn accepts_empty_rules() {
        let policy = ProviderCooldownPolicy {
            window_seconds: 0,
            rules: Vec::new(),
        };

        assert!(validate_provider_cooldown_policy(Some(&policy)).is_ok());
    }

    #[test]
    fn accepts_single_code_and_range_rules() {
        assert!(
            validate_provider_cooldown_policy(Some(&ProviderCooldownPolicy {
                window_seconds: 60,
                rules: vec![single_code_rule(429, 2, 60), range_rule(502, 504, 3, 120)],
            }))
            .is_ok()
        );
    }

    #[test]
    fn rejects_invalid_status_code_boundary() {
        assert_invalid_policy(
            range_policy(99, 101, 2, 60),
            "invalid input: provider_cooldown_policy status code range must be between 100 and 599",
        );
    }

    #[test]
    fn rejects_reversed_status_code_range() {
        assert_invalid_policy(
            range_policy(504, 502, 2, 60),
            "invalid input: provider_cooldown_policy.status_code_start must be less than or equal to status_code_end",
        );
    }

    #[test]
    fn rejects_overlapping_status_code_ranges() {
        let policy = ProviderCooldownPolicy {
            window_seconds: 60,
            rules: vec![range_rule(502, 504, 2, 60), range_rule(504, 505, 3, 120)],
        };

        assert_invalid_policy(
            policy,
            "invalid input: provider_cooldown_policy contains overlapping status code ranges: 502-504 and 504-505",
        );
    }

    #[test]
    fn rejects_non_positive_values() {
        assert_invalid_policy(
            single_code_policy(429, 0, 60),
            "invalid input: provider_cooldown_policy.failure_count must be greater than 0",
        );
    }

    fn range_policy(start: i32, end: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownPolicy {
        ProviderCooldownPolicy {
            window_seconds: 60,
            rules: vec![range_rule(start, end, failure_count, cooldown_seconds)],
        }
    }

    fn range_rule(start: i32, end: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownRule {
        ProviderCooldownRule {
            status_code_start: start,
            status_code_end: end,
            failure_count,
            cooldown_seconds,
        }
    }

    fn single_code_policy(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownPolicy {
        range_policy(status_code, status_code, failure_count, cooldown_seconds)
    }

    fn single_code_rule(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownRule {
        range_rule(status_code, status_code, failure_count, cooldown_seconds)
    }

    fn assert_invalid_policy(policy: ProviderCooldownPolicy, expected: &str) {
        let error = validate_provider_cooldown_policy(Some(&policy)).unwrap_err();

        assert_eq!(error.to_string(), expected);
    }
}
