use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rust_decimal::Decimal;
use types::{
    pagination::{Page, PageRequest},
    wallet::{Wallet, WalletAdjustment, WalletBalanceType, WalletId, WalletTransaction},
};

use crate::application::{WalletError, WalletRepository, WalletResult, WalletService, WalletUseCase};

#[tokio::test]
async fn balance_creates_cny_wallet_for_user() {
    let repository = MemoryWalletRepository::default();
    let service = WalletService::new(repository.clone());

    let response = service.balance("user-1").await.unwrap();

    assert_eq!(response.currency, "CNY");
    assert_eq!(response.balance, Decimal::ZERO);
    assert_eq!(repository.wallets().len(), 1);
}

#[tokio::test]
async fn transactions_return_wallet_summary_and_page() {
    let repository = MemoryWalletRepository::default();
    repository.insert_wallet(wallet("wallet-1", "user-1", Decimal::new(20, 0), Decimal::new(5, 0)));
    repository.insert_transaction(transaction("tx-1", "wallet-1", Decimal::new(3, 0)));
    let service = WalletService::new(repository);

    let response = service.transactions("user-1", PageRequest { page: 1, page_size: 20 }).await.unwrap();

    assert_eq!(response.total, 1);
    assert_eq!(response.wallet.balance, Decimal::new(25, 0));
    assert_eq!(response.items[0].id, "tx-1");
}

#[tokio::test]
async fn adjust_recharge_balance_records_snapshots() {
    let repository = MemoryWalletRepository::default();
    repository.insert_wallet(wallet("wallet-1", "user-1", Decimal::new(10, 0), Decimal::new(2, 0)));
    let service = WalletService::new(repository.clone());

    let tx = service
        .adjust_wallet(WalletAdjustment {
            wallet_id: "wallet-1".into(),
            amount: Decimal::new(3, 0),
            balance_type: WalletBalanceType::Recharge,
            operator_id: Some("admin-1".into()),
            description: Some("manual".into()),
        })
        .await
        .unwrap();

    assert_eq!(repository.wallets()[0].recharge_balance, Decimal::new(13, 0));
    assert_eq!(tx.balance_before, Decimal::new(12, 0));
    assert_eq!(tx.balance_after, Decimal::new(15, 0));
    assert_eq!(tx.recharge_balance_before, Decimal::new(10, 0));
    assert_eq!(tx.recharge_balance_after, Decimal::new(13, 0));
}

#[tokio::test]
async fn adjust_rejects_negative_gift_balance() {
    let repository = MemoryWalletRepository::default();
    repository.insert_wallet(wallet("wallet-1", "user-1", Decimal::ZERO, Decimal::new(2, 0)));
    let service = WalletService::new(repository);

    let result = service
        .adjust_wallet(WalletAdjustment {
            wallet_id: "wallet-1".into(),
            amount: Decimal::new(-3, 0),
            balance_type: WalletBalanceType::Gift,
            operator_id: None,
            description: None,
        })
        .await;

    assert!(matches!(result, Err(WalletError::InvalidInput(_))));
}

#[derive(Clone, Default)]
struct MemoryWalletRepository {
    state: Arc<Mutex<MemoryState>>,
}

#[derive(Default)]
struct MemoryState {
    wallets: Vec<Wallet>,
    transactions: Vec<WalletTransaction>,
}

impl MemoryWalletRepository {
    fn insert_wallet(&self, wallet: Wallet) {
        self.state.lock().unwrap().wallets.push(wallet);
    }

    fn insert_transaction(&self, transaction: WalletTransaction) {
        self.state.lock().unwrap().transactions.push(transaction);
    }

    fn wallets(&self) -> Vec<Wallet> {
        self.state.lock().unwrap().wallets.clone()
    }
}

#[async_trait]
impl WalletRepository for MemoryWalletRepository {
    async fn find_by_user_id(&self, user_id: &str) -> WalletResult<Option<Wallet>> {
        Ok(self.state.lock().unwrap().wallets.iter().find(|wallet| wallet.user_id == user_id).cloned())
    }

    async fn find_by_id(&self, wallet_id: &str) -> WalletResult<Option<Wallet>> {
        Ok(self.state.lock().unwrap().wallets.iter().find(|wallet| wallet.id.0 == wallet_id).cloned())
    }

    async fn create_user_wallet(&self, user_id: &str) -> WalletResult<Wallet> {
        let wallet = wallet(
            &format!("wallet-{}", self.state.lock().unwrap().wallets.len() + 1),
            user_id,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        self.insert_wallet(wallet.clone());
        Ok(wallet)
    }

    async fn save_ledger_entry(&self, wallet: Wallet, mut transaction: WalletTransaction) -> WalletResult<WalletTransaction> {
        let mut state = self.state.lock().unwrap();
        let index = state.wallets.iter().position(|item| item.id == wallet.id).ok_or(WalletError::NotFound)?;
        state.wallets[index] = wallet.clone();
        transaction.id = format!("tx-{}", state.transactions.len() + 1);
        state.transactions.push(transaction.clone());
        Ok(transaction)
    }

    async fn page_transactions(&self, wallet_id: &str, page: PageRequest) -> WalletResult<Page<WalletTransaction>> {
        let items: Vec<_> = self
            .state
            .lock()
            .unwrap()
            .transactions
            .iter()
            .filter(|transaction| transaction.wallet_id == wallet_id)
            .cloned()
            .collect();
        Ok(Page {
            total: items.len() as u64,
            items,
            page: page.page,
            page_size: page.page_size,
        })
    }
}

fn wallet(id: &str, user_id: &str, recharge_balance: Decimal, gift_balance: Decimal) -> Wallet {
    Wallet {
        id: WalletId(id.into()),
        user_id: user_id.into(),
        recharge_balance,
        gift_balance,
        currency: "CNY".into(),
        status: "active".into(),
        limit_mode: "finite".into(),
        total_recharged: Decimal::ZERO,
        total_consumed: Decimal::ZERO,
        total_refunded: Decimal::ZERO,
        total_adjusted: Decimal::ZERO,
        created_at: "2026-05-08T00:00:00Z".into(),
        updated_at: "2026-05-08T00:00:00Z".into(),
    }
}

fn transaction(id: &str, wallet_id: &str, amount: Decimal) -> WalletTransaction {
    WalletTransaction {
        id: id.into(),
        wallet_id: wallet_id.into(),
        category: "adjust".into(),
        reason_code: "adjust_admin".into(),
        amount,
        balance_before: Decimal::ZERO,
        balance_after: amount,
        recharge_balance_before: Decimal::ZERO,
        recharge_balance_after: amount,
        gift_balance_before: Decimal::ZERO,
        gift_balance_after: Decimal::ZERO,
        link_type: None,
        link_id: None,
        operator_id: None,
        description: None,
        created_at: "2026-05-08T00:00:00Z".into(),
    }
}
