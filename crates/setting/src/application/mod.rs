mod email_config;
mod error;
mod ports;
mod service;
mod smtp;
mod validation;

pub use error::{SettingError, SettingResult};
pub use ports::{SettingRepository, SettingSecretCipher, SettingUseCase, SettingUserGroupCatalog, SmtpConnectionTester};
pub use service::SettingService;
pub use smtp::{SmtpConnectionConfig, StoredSmtpSettings};
