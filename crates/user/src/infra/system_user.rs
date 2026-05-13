use configuration::Settings;
use types::user::{USER_QUOTA_MODE_WALLET, User, UserId, default_user_created_at};

use crate::application::{SystemUserProvider, SystemUserRecord};

#[derive(Clone)]
pub struct ConfigSystemUserProvider {
    record: SystemUserRecord,
}

impl ConfigSystemUserProvider {
    pub fn from_settings(settings: &Settings) -> Result<Self, configuration::SettingsError> {
        Ok(Self {
            record: SystemUserRecord {
                user: User {
                    id: UserId(settings.admin.id.trim().into()),
                    username: settings.admin.username.trim().into(),
                    email: settings.admin.email.trim().into(),
                    role: settings.admin.role.trim().into(),
                    is_active: settings.admin.is_active,
                    allowed_model_ids: Vec::new(),
                    allowed_provider_ids: Vec::new(),
                    auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                    email_verified: true,
                    system: true,
                    rate_limit_rpm: None,
                    quota_mode: USER_QUOTA_MODE_WALLET.into(),
                    created_at: default_user_created_at(),
                    last_login_at: None,
                },
                password_hash: settings.admin_password_hash()?,
            },
        })
    }
}

impl SystemUserProvider for ConfigSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        Some(self.record.clone())
    }
}
