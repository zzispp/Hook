pub(super) fn capability_list_enabled(capabilities: Option<&[String]>, required: &str) -> bool {
    let required = required.trim();
    capabilities.is_some_and(|items| items.iter().any(|value| value.eq_ignore_ascii_case(required)))
}
