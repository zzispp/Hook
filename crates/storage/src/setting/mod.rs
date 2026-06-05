mod record;
mod repository;
mod types;

pub use repository::{SYSTEM_SETTINGS_ID, SettingStore};
pub use types::{SystemSettingsAuthProviderRecord, SystemSettingsRecordPatch, SystemSettingsSmtpRecord};

pub(crate) use record::system_settings as system_setting_records;
