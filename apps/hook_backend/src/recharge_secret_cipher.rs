use recharge::application::{RechargeError, RechargeResult, RechargeSecretCipher};
use setting::{application::SettingSecretCipher, infra::SettingAesSecretCipher};

#[derive(Clone)]
pub(crate) struct RechargeAesSecretCipher {
    inner: SettingAesSecretCipher,
}

impl RechargeAesSecretCipher {
    pub(crate) const fn new(inner: SettingAesSecretCipher) -> Self {
        Self { inner }
    }
}

impl RechargeSecretCipher for RechargeAesSecretCipher {
    fn encrypt_secret(&self, plaintext: &str) -> RechargeResult<String> {
        self.inner.encrypt_secret(plaintext).map_err(setting_error)
    }

    fn decrypt_secret(&self, ciphertext: &str) -> RechargeResult<String> {
        self.inner.decrypt_secret(ciphertext).map_err(setting_error)
    }
}

fn setting_error(error: setting::application::SettingError) -> RechargeError {
    RechargeError::Infrastructure(error.to_string())
}
