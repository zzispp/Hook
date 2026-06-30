use rust_decimal::Decimal;

use crate::application::{UpstreamGroupRatio, UpstreamSyncSnapshot};

pub(super) enum GroupRatioLookup {
    Fixed(Decimal),
    Missing,
    NonFixed(String),
}

pub(super) fn group_ratio(snapshot: &UpstreamSyncSnapshot, group_id: Option<&str>, group: Option<&str>) -> GroupRatioLookup {
    let Some(group_key) = stable_group_key(group_id, group) else {
        return GroupRatioLookup::Missing;
    };
    match snapshot.groups.get(group_key) {
        Some(UpstreamGroupRatio::Fixed(value)) => GroupRatioLookup::Fixed(*value),
        Some(UpstreamGroupRatio::UpstreamValue(value)) => GroupRatioLookup::NonFixed(value.clone()),
        None => GroupRatioLookup::Missing,
    }
}

pub(super) fn same_group(current_group_id: Option<&str>, current_group: Option<&str>, observed_group_id: Option<&str>, observed_group: Option<&str>) -> bool {
    match (current_group_id, observed_group_id) {
        (Some(current), Some(observed)) => current == observed,
        _ => current_group == observed_group,
    }
}

fn stable_group_key<'a>(group_id: Option<&'a str>, group: Option<&'a str>) -> Option<&'a str> {
    group_id.or(group)
}
