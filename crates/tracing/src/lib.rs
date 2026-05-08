use std::{fmt::Display, sync::OnceLock, time::Duration};

use tracing_subscriber::{
    filter::{LevelFilter, LevelParseError},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

static GLOBAL_SUBSCRIBER_INITIALIZED: OnceLock<()> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TracingConfig {
    pub log_level: String,
}

/// Initialize the global tracing subscriber.
pub fn init_global_subscriber(config: TracingConfig) -> Result<(), LevelParseError> {
    let filter = parse_log_level(&config.log_level)?;
    GLOBAL_SUBSCRIBER_INITIALIZED.get_or_init(|| {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().with_target(false))
            .with(filter)
            .init();
    });
    Ok(())
}

fn parse_log_level(value: &str) -> Result<LevelFilter, LevelParseError> {
    value.parse()
}

/// Duration wrapper that prints milliseconds.
pub struct DurationMs(pub Duration);

impl Display for DurationMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0.as_millis())
    }
}

fn field_pairs(fields: &[(&str, &dyn Display)]) -> Option<String> {
    if fields.is_empty() {
        return None;
    }

    Some(fields.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join(" "))
}

pub fn info_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::info!("{message} {field_pairs}"),
        None => tracing::info!("{message}"),
    }
}

pub fn warn_with_fields_impl(message: &str, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::warn!("{message} {field_pairs}"),
        None => tracing::warn!("{message}"),
    }
}

pub fn error_with_fields_impl<E: std::error::Error + ?Sized>(message: &str, error: &E, fields: &[(&str, &dyn Display)]) {
    match field_pairs(fields) {
        Some(field_pairs) => tracing::error!("{message}: {field_pairs} {error}"),
        None => tracing::error!("{message}: {error}"),
    }
}

#[macro_export]
macro_rules! info_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::info_with_fields_impl($message, fields);
    }};
}

#[macro_export]
macro_rules! warn_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::warn_with_fields_impl($message, fields);
    }};
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $error:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::error_with_fields_impl($message, $error, fields);
    }};
}

pub fn error<E: std::error::Error + ?Sized>(message: &str, error: &E) {
    tracing::error!("{message}: {error}");
}

#[cfg(test)]
mod tests {
    use super::{DurationMs, TracingConfig};
    use std::time::Duration;

    #[test]
    fn duration_ms_formats_milliseconds() {
        assert_eq!(DurationMs(Duration::from_millis(42)).to_string(), "42ms");
    }

    #[test]
    fn tracing_config_rejects_invalid_log_level() {
        let config = TracingConfig {
            log_level: "not a level".into(),
        };

        let result = super::init_global_subscriber(config);

        assert!(result.is_err());
    }

    #[test]
    fn parse_log_level_accepts_standard_level() {
        assert_eq!(super::parse_log_level("debug").unwrap(), tracing_subscriber::filter::LevelFilter::DEBUG);
    }
}
