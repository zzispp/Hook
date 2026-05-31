mod email_config;
mod error;
mod oauth_config;
mod ports;
mod service;
mod smtp;
mod validation;
mod wallet_config;

pub use error::{SettingError, SettingResult};
pub use ports::{SettingPaymentChannelCatalog, SettingRepository, SettingSecretCipher, SettingUseCase, SettingUserGroupCatalog, SmtpConnectionTester};
pub use service::SettingService;
pub use smtp::{SmtpConnectionConfig, StoredSmtpSettings};
