use std::collections::HashSet;

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
    let mut status_codes = HashSet::new();
    for rule in &policy.rules {
        validate_status_code(rule.status_code)?;
        validate_positive_value("provider_cooldown_policy.failure_count", rule.failure_count)?;
        validate_positive_value("provider_cooldown_policy.cooldown_seconds", rule.cooldown_seconds)?;
        if !status_codes.insert(rule.status_code) {
            return Err(SettingError::InvalidInput(format!(
                "provider_cooldown_policy contains duplicate status_code: {}",
                rule.status_code
            )));
        }
    }
    Ok(())
}

fn validate_positive_value(field: &str, value: i64) -> SettingResult<()> {
    if value <= 0 {
        return Err(SettingError::InvalidInput(format!("{field} must be greater than 0")));
    }
    Ok(())
}

fn validate_status_code(value: i32) -> SettingResult<()> {
    if !(MIN_STATUS_CODE..=MAX_STATUS_CODE).contains(&value) {
        return Err(SettingError::InvalidInput(format!(
            "provider_cooldown_policy.status_code must be between {MIN_STATUS_CODE} and {MAX_STATUS_CODE}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use types::provider::{ProviderCooldownPolicy, ProviderCooldownRule};

    use super::validate_provider_cooldown_policy;

    #[test]
    fn validate_provider_cooldown_policy_accepts_empty_rules() {
        let policy = ProviderCooldownPolicy {
            window_seconds: 0,
            rules: Vec::new(),
        };

        assert!(validate_provider_cooldown_policy(Some(&policy)).is_ok());
    }

    #[test]
    fn validate_provider_cooldown_policy_rejects_invalid_status_code() {
        let policy = policy_with_rule(99, 2, 60);
        let error = validate_provider_cooldown_policy(Some(&policy)).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: provider_cooldown_policy.status_code must be between 100 and 599"
        );
    }

    #[test]
    fn validate_provider_cooldown_policy_rejects_duplicate_status_code() {
        let policy = ProviderCooldownPolicy {
            window_seconds: 60,
            rules: vec![cooldown_rule(429, 2, 60), cooldown_rule(429, 3, 120)],
        };
        let error = validate_provider_cooldown_policy(Some(&policy)).unwrap_err();

        assert_eq!(error.to_string(), "invalid input: provider_cooldown_policy contains duplicate status_code: 429");
    }

    #[test]
    fn validate_provider_cooldown_policy_rejects_non_positive_values() {
        let policy = policy_with_rule(429, 0, 60);
        let error = validate_provider_cooldown_policy(Some(&policy)).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid input: provider_cooldown_policy.failure_count must be greater than 0"
        );
    }

    fn policy_with_rule(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownPolicy {
        ProviderCooldownPolicy {
            window_seconds: 60,
            rules: vec![cooldown_rule(status_code, failure_count, cooldown_seconds)],
        }
    }

    fn cooldown_rule(status_code: i32, failure_count: i64, cooldown_seconds: i64) -> ProviderCooldownRule {
        ProviderCooldownRule {
            status_code,
            failure_count,
            cooldown_seconds,
        }
    }
}
