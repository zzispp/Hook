use crate::application::{ProviderQuickImportSyncKey, ProviderQuickImportSyncSource};

pub(super) fn source_label(source: &ProviderQuickImportSyncSource) -> String {
    format!("{}({})", source.provider_name, source.provider_id)
}

pub(super) fn key_label(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey) -> String {
    format!(
        "提供商 {} / 本地密钥 {}({}) / 上游令牌 {}({})",
        source_label(source),
        key.local_key_name,
        key.key_id,
        key.upstream_token_name,
        key.upstream_token_id
    )
}

pub(super) fn key_detail(source: &ProviderQuickImportSyncSource, key: &ProviderQuickImportSyncKey, detail: String) -> String {
    format!("上下文：{}。{}", key_label(source, key), detail)
}
