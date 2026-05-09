use std::sync::Arc;

use crate::application::WalletUseCase;

#[derive(Clone)]
pub struct WalletApiState {
    pub wallets: Arc<dyn WalletUseCase>,
}

impl WalletApiState {
    pub fn new(wallets: Arc<dyn WalletUseCase>) -> Self {
        Self { wallets }
    }
}
