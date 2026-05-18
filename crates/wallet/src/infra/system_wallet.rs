use configuration::{Settings, SettingsError};
use rust_decimal::Decimal;
use types::{
    user::default_user_created_at,
    wallet::{Wallet, WalletId},
};

use crate::application::{SystemWalletProvider, SystemWalletRecord};

#[derive(Clone)]
pub struct ConfigSystemWalletProvider {
    record: SystemWalletRecord,
}

impl ConfigSystemWalletProvider {
    pub fn from_settings(settings: &Settings) -> Result<Self, SettingsError> {
        Ok(Self {
            record: SystemWalletRecord {
                wallet: Wallet {
                    id: WalletId(required_admin_wallet_value("admin.wallet.id", &settings.admin.wallet.id)?),
                    user_id: required_admin_wallet_value("admin.id", &settings.admin.id)?,
                    recharge_balance: Decimal::ZERO,
                    gift_balance: Decimal::ZERO,
                    currency: currency::DEFAULT_WALLET_CURRENCY.into(),
                    status: required_admin_wallet_value("admin.wallet.status", &settings.admin.wallet.status)?,
                    limit_mode: required_admin_wallet_value("admin.wallet.limit_mode", &settings.admin.wallet.limit_mode)?,
                    total_recharged: Decimal::ZERO,
                    total_consumed: Decimal::ZERO,
                    total_refunded: Decimal::ZERO,
                    total_adjusted: Decimal::ZERO,
                    created_at: default_user_created_at(),
                    updated_at: default_user_created_at(),
                },
                owner_name: required_admin_wallet_value("admin.username", &settings.admin.username)?,
                owner_email: required_admin_wallet_value("admin.email", &settings.admin.email)?,
            },
        })
    }
}

impl SystemWalletProvider for ConfigSystemWalletProvider {
    fn system_wallet(&self) -> Option<SystemWalletRecord> {
        Some(self.record.clone())
    }
}

fn required_admin_wallet_value(key: &'static str, value: &str) -> Result<String, SettingsError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }
    Ok(value.to_owned())
}
