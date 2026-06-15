use regex::Regex;

use crate::llm_proxy::LlmProxyError;

use super::CircuitAttemptEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CircuitScope {
    Route,
    Key,
}

#[derive(Clone, Debug)]
pub(super) struct RoutingCircuitRule {
    pub(super) id: &'static str,
    pub(super) scope: CircuitScope,
    pub(super) status_code_range: Option<(u16, u16)>,
    pub(super) error_type: Option<&'static str>,
    pub(super) error_code: Option<&'static str>,
    pub(super) error_message_regex: Option<&'static str>,
    pub(super) failure_count: u64,
    pub(super) min_sample_count: u64,
    pub(super) window_seconds: i64,
    pub(super) cooldown_seconds: i64,
    pub(super) half_open_success_count: u64,
}

pub(super) fn default_rules() -> Vec<RoutingCircuitRule> {
    vec![
        rule(RuleInput::new("http_429_key", CircuitScope::Key).status(429, 429).counts(3, 3).times(60, 120)),
        rule(
            RuleInput::new("http_5xx_route", CircuitScope::Route)
                .status(500, 599)
                .counts(4, 4)
                .times(120, 180),
        ),
        rule(
            RuleInput::new("timeout_route", CircuitScope::Route)
                .error_type("timeout")
                .counts(2, 2)
                .times(120, 180),
        ),
        regex_rule(),
    ]
}

impl RoutingCircuitRule {
    pub(super) fn matches(&self, event: &CircuitAttemptEvent<'_>) -> Result<bool, LlmProxyError> {
        if !self.matches_status(event) || !self.matches_text(event)? {
            return Ok(false);
        }
        Ok(self.status_code_range.is_some() || self.error_type.is_some() || self.error_code.is_some() || self.error_message_regex.is_some())
    }

    fn matches_status(&self, event: &CircuitAttemptEvent<'_>) -> bool {
        let status_ok = self
            .status_code_range
            .is_none_or(|(start, end)| event.status_code.is_some_and(|code| code >= i32::from(start) && code <= i32::from(end)));
        let type_ok = self.error_type.is_none_or(|value| event.error_type == Some(value));
        let code_ok = self.error_code.is_none_or(|value| event.error_code == Some(value));
        status_ok && type_ok && code_ok
    }

    fn matches_text(&self, event: &CircuitAttemptEvent<'_>) -> Result<bool, LlmProxyError> {
        let Some(pattern) = self.error_message_regex else {
            return Ok(true);
        };
        let regex = Regex::new(pattern).map_err(|error| LlmProxyError::Infrastructure(format!("routing circuit regex error: {error}")))?;
        Ok(event.error_message.is_some_and(|message| regex.is_match(message)))
    }
}

#[derive(Clone, Copy)]
struct RuleInput {
    id: &'static str,
    scope: CircuitScope,
    failure_count: u64,
    min_sample_count: u64,
    window_seconds: i64,
    cooldown_seconds: i64,
    status_code_range: Option<(u16, u16)>,
    error_type: Option<&'static str>,
}

impl RuleInput {
    const fn new(id: &'static str, scope: CircuitScope) -> Self {
        Self {
            id,
            scope,
            failure_count: 1,
            min_sample_count: 1,
            window_seconds: 60,
            cooldown_seconds: 120,
            status_code_range: None,
            error_type: None,
        }
    }

    const fn status(mut self, start: u16, end: u16) -> Self {
        self.status_code_range = Some((start, end));
        self
    }

    const fn error_type(mut self, value: &'static str) -> Self {
        self.error_type = Some(value);
        self
    }

    const fn counts(mut self, failure_count: u64, min_sample_count: u64) -> Self {
        self.failure_count = failure_count;
        self.min_sample_count = min_sample_count;
        self
    }

    const fn times(mut self, window_seconds: i64, cooldown_seconds: i64) -> Self {
        self.window_seconds = window_seconds;
        self.cooldown_seconds = cooldown_seconds;
        self
    }
}

fn rule(input: RuleInput) -> RoutingCircuitRule {
    RoutingCircuitRule {
        id: input.id,
        scope: input.scope,
        status_code_range: input.status_code_range,
        error_type: input.error_type,
        error_code: None,
        error_message_regex: None,
        failure_count: input.failure_count,
        min_sample_count: input.min_sample_count,
        window_seconds: input.window_seconds,
        cooldown_seconds: input.cooldown_seconds,
        half_open_success_count: 2,
    }
}

fn regex_rule() -> RoutingCircuitRule {
    let mut rule = rule(RuleInput::new("overload_message_route", CircuitScope::Route).counts(3, 3).times(120, 180));
    rule.error_message_regex = Some("(?i)(overload|temporarily unavailable|upstream unavailable)");
    rule
}
