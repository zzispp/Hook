use rust_decimal::Decimal;

use crate::application::{UpstreamGroupRatio, UpstreamSyncSnapshot};

pub(super) enum GroupRatioLookup {
    Fixed(Decimal),
    Missing,
    NonFixed(String),
}

pub(super) fn group_ratio(snapshot: &UpstreamSyncSnapshot, group: Option<&str>) -> GroupRatioLookup {
    let Some(group) = group else {
        return GroupRatioLookup::Missing;
    };
    match snapshot.groups.get(group) {
        Some(UpstreamGroupRatio::Fixed(value)) => GroupRatioLookup::Fixed(*value),
        Some(UpstreamGroupRatio::UpstreamValue(value)) => GroupRatioLookup::NonFixed(value.clone()),
        None => GroupRatioLookup::Missing,
    }
}
