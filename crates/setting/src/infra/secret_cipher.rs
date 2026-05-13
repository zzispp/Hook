use ring::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use sha2::{Digest, Sha256};

use crate::application::{SettingError, SettingResult, SettingSecretCipher};

const CIPHER_VERSION: &str = "v1";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Clone)]
pub struct SettingAesSecretCipher {
    key: [u8; KEY_LEN],
}

impl SettingAesSecretCipher {
    pub fn new(secret: String) -> SettingResult<Self> {
        let secret = secret.trim();
        if secret.is_empty() {
            return Err(secret_error("setting secret encryption key cannot be blank"));
        }
        Ok(Self { key: derived_key(secret) })
    }

    fn encryption_key(&self) -> SettingResult<LessSafeKey> {
        let key = UnboundKey::new(&AES_256_GCM, &self.key).map_err(|_| secret_error("setting secret cipher initialization failed"))?;
        Ok(LessSafeKey::new(key))
    }
}

impl SettingSecretCipher for SettingAesSecretCipher {
    fn encrypt_secret(&self, plaintext: &str) -> SettingResult<String> {
        let nonce = random_nonce()?;
        let mut ciphertext = plaintext.as_bytes().to_vec();
        self.encryption_key()?
            .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut ciphertext)
            .map_err(|_| secret_error("setting secret encryption failed"))?;
        Ok(format!("{CIPHER_VERSION}:{}:{}", hex::encode(nonce), hex::encode(ciphertext)))
    }

    fn decrypt_secret(&self, ciphertext: &str) -> SettingResult<String> {
        let (nonce, mut encrypted) = parse_ciphertext(ciphertext)?;
        let plaintext = self
            .encryption_key()?
            .open_in_place(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut encrypted)
            .map_err(|_| secret_error("setting secret decryption failed"))?;
        String::from_utf8(plaintext.to_vec()).map_err(|_| secret_error("setting secret plaintext is not valid utf-8"))
    }
}

fn derived_key(secret: &str) -> [u8; KEY_LEN] {
    let digest = Sha256::digest(secret.as_bytes());
    let mut key = [0_u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}

fn random_nonce() -> SettingResult<[u8; NONCE_LEN]> {
    let mut nonce = [0_u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| secret_error("setting secret nonce generation failed"))?;
    Ok(nonce)
}

fn parse_ciphertext(value: &str) -> SettingResult<([u8; NONCE_LEN], Vec<u8>)> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != CIPHER_VERSION {
        return Err(secret_error("setting secret ciphertext format is invalid"));
    }
    let nonce = parse_nonce(parts[1])?;
    let ciphertext = hex::decode(parts[2]).map_err(|_| secret_error("setting secret ciphertext is not hex"))?;
    Ok((nonce, ciphertext))
}

fn parse_nonce(value: &str) -> SettingResult<[u8; NONCE_LEN]> {
    let bytes = hex::decode(value).map_err(|_| secret_error("setting secret nonce is not hex"))?;
    bytes.try_into().map_err(|_| secret_error("setting secret nonce length is invalid"))
}

fn secret_error(message: &'static str) -> SettingError {
    SettingError::Infrastructure(message.into())
}

#[cfg(test)]
mod tests {
    use crate::{
        application::SettingSecretCipher,
        infra::secret_cipher::{CIPHER_VERSION, NONCE_LEN, SettingAesSecretCipher},
    };

    #[test]
    fn encrypt_secret_returns_versioned_ciphertext() {
        let cipher = SettingAesSecretCipher::new("setting-secret".into()).unwrap();

        let encrypted = cipher.encrypt_secret("smtp-password").unwrap();

        let parts = encrypted.split(':').collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], CIPHER_VERSION);
        assert_eq!(hex::decode(parts[1]).unwrap().len(), NONCE_LEN);
        assert!(hex::decode(parts[2]).unwrap().len() > "smtp-password".len());
        assert_ne!(encrypted, "smtp-password");
    }

    #[test]
    fn encrypt_secret_uses_unique_nonce() {
        let cipher = SettingAesSecretCipher::new("setting-secret".into()).unwrap();

        let first = cipher.encrypt_secret("smtp-password").unwrap();
        let second = cipher.encrypt_secret("smtp-password").unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn decrypt_secret_returns_original_plaintext() {
        let cipher = SettingAesSecretCipher::new("setting-secret".into()).unwrap();
        let encrypted = cipher.encrypt_secret("smtp-password").unwrap();

        let decrypted = cipher.decrypt_secret(&encrypted).unwrap();

        assert_eq!(decrypted, "smtp-password");
    }

    #[test]
    fn decrypt_secret_rejects_invalid_ciphertext() {
        let cipher = SettingAesSecretCipher::new("setting-secret".into()).unwrap();

        let result = cipher.decrypt_secret("smtp-password");

        assert!(result.is_err());
    }
}
