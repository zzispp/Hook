use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn format_timestamp(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).expect("operations timestamp must format as RFC3339")
}
