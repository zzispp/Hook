use rust_decimal::Decimal;
use types::provider::ProviderQuickImportSyncStatus;

use crate::application::{ProviderError, ProviderQuickImportSyncEventCreate, ProviderQuickImportSyncKey, ProviderQuickImportSyncSource};

use super::{
    quick_import_sync_event_labels::{key_detail, key_label, source_label},
    quick_import_sync_outcome::KeyOutcome,
};

pub(super) fn key_events(
    source: &ProviderQuickImportSyncSource,
    key: &ProviderQuickImportSyncKey,
    outcome: &KeyOutcome,
) -> Vec<ProviderQuickImportSyncEventCreate> {
    let mut events = Vec::new();
    if let Some(event) = group_sync_event(source, key, outcome) {
        events.push(event);
    }
    if let Some(event) = anomaly_event(source, key, outcome) {
        events.push(event);
    }
    if let Some(event) = cost_event(source, key, outcome) {
        events.push(event);
    }
    if let Some(event) = model_candidate_event(source, key, outcome) {
        events.push(event);
    }
    events
}

pub(super) fn source_failure_event(source: &ProviderQuickImportSyncSource, error: &ProviderError, disable: bool) -> ProviderQuickImportSyncEventCreate {
    let action = if disable {
        "连续失败已达到阈值，将禁用相关密钥"
    } else {
        "仅记录失败，不禁用密钥"
    };
    event(
        source,
        None,
        ProviderQuickImportSyncStatus::SourceFetchFailed,
        format!("快捷导入同步失败：提供商 {}", source_label(source)),
        format!("同步来源拉取失败：{}。{}", error, action),
    )
}

pub(super) fn source_failure_key_event(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey) -> ProviderQuickImportSyncEventCreate {
    event(
        source,
        Some(key),
        ProviderQuickImportSyncStatus::SourceFetchFailed,
        format!("快捷导入同步：{} 已因连续拉取失败禁用", key_label(source, key)),
        key_detail(
            source,
            key,
            format!(
                "同步来源连续失败达到 {} 次，已按策略禁用本地密钥。",
                source.sync_config.fetch_failure_disable_threshold
            ),
        ),
    )
}

fn group_sync_event(
    source: &ProviderQuickImportSyncSource,
    key: &ProviderQuickImportSyncKey,
    outcome: &KeyOutcome,
) -> Option<ProviderQuickImportSyncEventCreate> {
    let observed = outcome.synced_group()?;
    if observed.as_deref() == key.upstream_group.as_deref() {
        return None;
    }
    Some(event(
        source,
        Some(key),
        ProviderQuickImportSyncStatus::UpstreamGroupChanged,
        format!("快捷导入同步：{} 上游分组已同步", key_label(source, key)),
        key_detail(
            source,
            key,
            format!(
                "上游令牌所属分组从 {} 变更为 {}，{}。",
                group_label(key.upstream_group.as_deref()),
                group_label(observed.as_deref()),
                group_sync_action(outcome)
            ),
        ),
    ))
}

fn anomaly_event(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey, outcome: &KeyOutcome) -> Option<ProviderQuickImportSyncEventCreate> {
    let status = outcome.statuses.iter().copied().find(anomaly_status)?;
    if key.statuses.contains(&status) {
        return None;
    }
    let action = if outcome.disable_key {
        "已按策略禁用本地密钥"
    } else {
        "仅记录异常，未禁用本地密钥"
    };
    Some(event(
        source,
        Some(key),
        status,
        anomaly_title(source, key, status),
        key_detail(source, key, format!("{}。{}", anomaly_detail(key, outcome, status), action)),
    ))
}

fn cost_event(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey, outcome: &KeyOutcome) -> Option<ProviderQuickImportSyncEventCreate> {
    let next = outcome.observed_effective_multiplier?;
    if next == key.effective_cost_multiplier || suppress_pending_cost_event(key, outcome) {
        return None;
    }
    let direction = multiplier_direction(key.effective_cost_multiplier, next);
    let pending = outcome.statuses.contains(&ProviderQuickImportSyncStatus::CostPendingUpdate);
    let action = if pending { "待更新为" } else { "已更新为" };
    let suffix = if pending { "当前成本同步模式为仅提示。" } else { "" };
    Some(event(
        source,
        Some(key),
        cost_event_status(outcome),
        format!("快捷导入同步：{} 成本倍率{}", key_label(source, key), direction),
        key_detail(
            source,
            key,
            format!(
                "原上游倍率 {}，最终成本倍率从 {} {} {}。{}",
                multiplier_label(outcome.observed_group_ratio),
                multiplier_label(Some(key.effective_cost_multiplier)),
                action,
                multiplier_label(Some(next)),
                suffix
            ),
        ),
    ))
}

fn model_candidate_event(
    source: &ProviderQuickImportSyncSource,
    key: &ProviderQuickImportSyncKey,
    outcome: &KeyOutcome,
) -> Option<ProviderQuickImportSyncEventCreate> {
    if outcome.candidate_model_ids.is_empty() || key.statuses.contains(&ProviderQuickImportSyncStatus::ModelCandidateAvailable) {
        return None;
    }
    Some(event(
        source,
        Some(key),
        ProviderQuickImportSyncStatus::ModelCandidateAvailable,
        format!("快捷导入同步：{} 发现可关联模型", key_label(source, key)),
        key_detail(
            source,
            key,
            format!(
                "上游令牌发现 {} 个可关联到同名全局模型的候选模型：{}。系统不会自动关联，请在密钥模型关联里确认。",
                outcome.candidate_model_ids.len(),
                outcome.candidate_model_ids.join("，")
            ),
        ),
    ))
}

fn event(
    source: &ProviderQuickImportSyncSource,
    key: Option<&ProviderQuickImportSyncKey>,
    status: ProviderQuickImportSyncStatus,
    title: String,
    detail: String,
) -> ProviderQuickImportSyncEventCreate {
    ProviderQuickImportSyncEventCreate {
        provider_id: source.provider_id.clone(),
        source_id: source.id.clone(),
        key_id: key.map(|value| value.key_id.clone()),
        status,
        title,
        detail,
    }
}

fn anomaly_status(status: &ProviderQuickImportSyncStatus) -> bool {
    matches!(
        status,
        ProviderQuickImportSyncStatus::UpstreamTokenDeleted
            | ProviderQuickImportSyncStatus::UpstreamTokenDisabled
            | ProviderQuickImportSyncStatus::UpstreamGroupRemoved
            | ProviderQuickImportSyncStatus::UpstreamGroupChanged
            | ProviderQuickImportSyncStatus::UpstreamKeyUnavailable
            | ProviderQuickImportSyncStatus::UpstreamModelRemoved
            | ProviderQuickImportSyncStatus::NoAssociatedModels
            | ProviderQuickImportSyncStatus::CostUnavailable
    )
}

fn anomaly_title(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey, status: ProviderQuickImportSyncStatus) -> String {
    format!("快捷导入同步异常：{} {}", key_label(source, key), anomaly_reason(status))
}

fn anomaly_reason(status: ProviderQuickImportSyncStatus) -> &'static str {
    match status {
        ProviderQuickImportSyncStatus::UpstreamTokenDeleted => "上游令牌被删除",
        ProviderQuickImportSyncStatus::UpstreamTokenDisabled => "上游令牌被禁用",
        ProviderQuickImportSyncStatus::UpstreamGroupRemoved => "上游分组被删除",
        ProviderQuickImportSyncStatus::UpstreamGroupChanged => "上游令牌所属分组变更",
        ProviderQuickImportSyncStatus::UpstreamKeyUnavailable => "上游模型列表拉取失败",
        ProviderQuickImportSyncStatus::UpstreamModelRemoved => "已导入模型在上游消失",
        ProviderQuickImportSyncStatus::NoAssociatedModels => "没有关联模型",
        ProviderQuickImportSyncStatus::CostUnavailable => "成本不可计算",
        _ => "同步异常",
    }
}

fn anomaly_detail(key: &ProviderQuickImportSyncKey, outcome: &KeyOutcome, status: ProviderQuickImportSyncStatus) -> String {
    match status {
        ProviderQuickImportSyncStatus::UpstreamGroupRemoved => format!("导入时分组 {} 已不存在", group_label(key.upstream_group.as_deref())),
        ProviderQuickImportSyncStatus::UpstreamGroupChanged => format!(
            "上游令牌所属分组从 {} 变更为 {}",
            group_label(key.upstream_group.as_deref()),
            group_label(outcome.observed_group.as_ref().and_then(|value| value.as_deref()))
        ),
        ProviderQuickImportSyncStatus::UpstreamKeyUnavailable => format!(
            "同步器已确认上游令牌仍存在且启用，但获取该令牌的裸 key 或请求 /v1/models 失败：{}",
            anomaly_error(outcome)
        ),
        ProviderQuickImportSyncStatus::CostUnavailable => format!("无法计算快捷导入成本：{}", anomaly_error(outcome)),
        ProviderQuickImportSyncStatus::NoAssociatedModels => "本地密钥没有任何快捷导入模型关联".into(),
        _ => anomaly_reason(status).into(),
    }
}

fn anomaly_error(outcome: &KeyOutcome) -> &str {
    outcome.error_message().unwrap_or("未返回具体错误")
}

fn suppress_pending_cost_event(key: &ProviderQuickImportSyncKey, outcome: &KeyOutcome) -> bool {
    outcome.statuses.contains(&ProviderQuickImportSyncStatus::CostPendingUpdate) && key.statuses.contains(&ProviderQuickImportSyncStatus::CostPendingUpdate)
}

fn cost_event_status(outcome: &KeyOutcome) -> ProviderQuickImportSyncStatus {
    if outcome.statuses.contains(&ProviderQuickImportSyncStatus::CostPendingUpdate) {
        ProviderQuickImportSyncStatus::CostPendingUpdate
    } else {
        ProviderQuickImportSyncStatus::Ok
    }
}

fn group_sync_action(outcome: &KeyOutcome) -> &'static str {
    if outcome.statuses.contains(&ProviderQuickImportSyncStatus::CostPendingUpdate) {
        return "系统已按策略接受新分组，成本同步模式为仅提示";
    }
    if outcome.costs.is_some() {
        return "系统已按策略接受新分组并覆盖成本";
    }
    "系统已按策略接受新分组"
}

fn multiplier_direction(previous: Decimal, next: Decimal) -> &'static str {
    if next > previous {
        "上涨"
    } else if next < previous {
        "下降"
    } else {
        "变化"
    }
}

fn multiplier_label(value: Option<Decimal>) -> String {
    value.map(|item| format!("{}x", item.normalize())).unwrap_or_else(|| "-".into())
}

fn group_label(value: Option<&str>) -> String {
    value.filter(|item| !item.is_empty()).unwrap_or("未设置").to_owned()
}
