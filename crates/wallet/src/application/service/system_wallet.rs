use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    wallet::{AdminWalletListFilters, AdminWalletResponse, AdminWalletTransactionsResponse, Wallet, WalletSummaryResponse, WalletTransactionsResponse},
};

use crate::application::SystemWalletRecord;

pub(super) fn wallet_for_user(record: Option<SystemWalletRecord>, user_id: &str) -> Option<Wallet> {
    record.filter(|record| record.wallet.user_id == user_id).map(|record| record.wallet)
}

pub(super) fn wallet_for_id(record: Option<SystemWalletRecord>, wallet_id: &str) -> Option<SystemWalletRecord> {
    record.filter(|record| record.wallet.id.0 == wallet_id)
}

pub(super) fn admin_wallet_response(record: SystemWalletRecord) -> AdminWalletResponse {
    let summary = WalletSummaryResponse::from(record.wallet.clone());
    AdminWalletResponse {
        id: summary.id,
        user_id: summary.user_id,
        owner_name: record.owner_name,
        owner_email: record.owner_email,
        owner_type: "system".into(),
        balance: summary.balance,
        recharge_balance: summary.recharge_balance,
        gift_balance: summary.gift_balance,
        currency: summary.currency,
        status: summary.status,
        limit_mode: summary.limit_mode,
        unlimited: summary.unlimited,
        total_recharged: summary.total_recharged,
        total_consumed: summary.total_consumed,
        total_refunded: summary.total_refunded,
        total_adjusted: summary.total_adjusted,
        created_at: record.wallet.created_at,
        updated_at: summary.updated_at,
    }
}

pub(super) fn system_wallet_matches(record: &SystemWalletRecord, filters: &AdminWalletListFilters) -> bool {
    status_matches(record, filters) && search_matches(record, filters)
}

pub(super) fn admin_wallet_slice(page: PageRequest, system_wallet_matches: bool) -> (PageSliceRequest, bool, u64) {
    let system_count = u64::from(system_wallet_matches);
    let global_offset = (page.page - 1) * page.page_size;
    let include_system = system_wallet_matches && global_offset == 0;
    let db_limit = page.page_size - u64::from(include_system);
    let slice = PageSliceRequest {
        offset: global_offset.saturating_sub(system_count),
        limit: db_limit.max(1),
        page: page.page,
        page_size: page.page_size,
    };
    (slice, include_system, db_limit)
}

pub(super) fn admin_wallet_page(
    page: PageRequest,
    db_page: Page<AdminWalletResponse>,
    system_wallet: Option<AdminWalletResponse>,
    include_system: bool,
    db_limit: u64,
) -> Page<AdminWalletResponse> {
    let system_count = u64::from(system_wallet.is_some());
    let mut items = Vec::new();
    if include_system && let Some(system_wallet) = system_wallet {
        items.push(system_wallet);
    }
    items.extend(db_page.items.into_iter().take(db_limit as usize));
    Page {
        items,
        total: db_page.total + system_count,
        page: page.page,
        page_size: page.page_size,
    }
}

pub(super) fn user_transactions_response(wallet: Wallet, page: PageRequest) -> WalletTransactionsResponse {
    WalletTransactionsResponse {
        wallet: WalletSummaryResponse::from(wallet),
        items: Vec::new(),
        total: 0,
        page: page.page,
        page_size: page.page_size,
    }
}

pub(super) fn admin_transactions_response(record: SystemWalletRecord, page: PageRequest) -> AdminWalletTransactionsResponse {
    AdminWalletTransactionsResponse {
        wallet: admin_wallet_response(record),
        items: Vec::new(),
        total: 0,
        page: page.page,
        page_size: page.page_size,
    }
}

fn status_matches(record: &SystemWalletRecord, filters: &AdminWalletListFilters) -> bool {
    filters
        .status
        .as_ref()
        .is_none_or(|status| status.is_empty() || record.wallet.status == *status)
}

fn search_matches(record: &SystemWalletRecord, filters: &AdminWalletListFilters) -> bool {
    let Some(search) = filters.search.as_ref().filter(|value| !value.is_empty()) else {
        return true;
    };
    record.wallet.id.0.contains(search) || record.wallet.user_id.contains(search) || record.owner_name.contains(search) || record.owner_email.contains(search)
}
