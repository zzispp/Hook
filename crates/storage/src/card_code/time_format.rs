use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub(super) fn parse_optional(value: Option<&str>) -> Option<OffsetDateTime> {
    value.map(parse_required)
}

pub(super) fn parse_required(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &Rfc3339).expect("stored timestamp must parse")
}
