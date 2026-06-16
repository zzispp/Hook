use rust_decimal::Decimal;
use types::provider::ProviderQuickImportSyncStatus;

const QUICK_IMPORT_MULTIPLIER_SCALE: u32 = 8;

pub(super) fn persisted_multiplier(value: Decimal) -> Decimal {
    value.round_dp(QUICK_IMPORT_MULTIPLIER_SCALE)
}

pub(super) fn statuses_with_candidates(mut statuses: Vec<ProviderQuickImportSyncStatus>, candidates: &[String]) -> Vec<ProviderQuickImportSyncStatus> {
    if !candidates.is_empty() {
        statuses.push(ProviderQuickImportSyncStatus::ModelCandidateAvailable);
    }
    statuses
}
