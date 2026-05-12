use ring::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};
use sha2::{Digest, Sha256};

use crate::application::{ProviderError, ProviderResult, SecretCipher};

const CIPHER_VERSION: &str = "v1";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Clone)]
pub struct ProviderKeyCipher {
    key: [u8; KEY_LEN],
}

impl ProviderKeyCipher {
    pub fn new(secret: String) -> ProviderResult<Self> {
        let secret = secret.trim();
        if secret.is_empty() {
            return Err(ProviderError::Secret("provider key encryption secret cannot be blank".into()));
        }
        Ok(Self { key: derived_key(secret) })
    }

    fn encryption_key(&self) -> ProviderResult<LessSafeKey> {
        let key = UnboundKey::new(&AES_256_GCM, &self.key).map_err(|_| ProviderError::Secret("provider key cipher initialization failed".into()))?;
        Ok(LessSafeKey::new(key))
    }
}

impl SecretCipher for ProviderKeyCipher {
    fn encrypt_provider_key(&self, plaintext: &str) -> ProviderResult<String> {
        let nonce = random_nonce()?;
        let mut ciphertext = plaintext.as_bytes().to_vec();
        self.encryption_key()?
            .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut ciphertext)
            .map_err(|_| ProviderError::Secret("provider key encryption failed".into()))?;
        Ok(format!("{CIPHER_VERSION}:{}:{}", hex::encode(nonce), hex::encode(ciphertext)))
    }

    fn decrypt_provider_key(&self, ciphertext: &str) -> ProviderResult<String> {
        let (nonce, mut encrypted) = parse_ciphertext(ciphertext)?;
        let plaintext = self
            .encryption_key()?
            .open_in_place(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut encrypted)
            .map_err(|_| ProviderError::Secret("provider key decryption failed".into()))?;
        String::from_utf8(plaintext.to_vec()).map_err(|_| ProviderError::Secret("provider key plaintext is not valid utf-8".into()))
    }
}

fn derived_key(secret: &str) -> [u8; KEY_LEN] {
    let digest = Sha256::digest(secret.as_bytes());
    let mut key = [0_u8; KEY_LEN];
    key.copy_from_slice(&digest);
    key
}

fn random_nonce() -> ProviderResult<[u8; NONCE_LEN]> {
    let mut nonce = [0_u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce)
        .map_err(|_| ProviderError::Secret("provider key nonce generation failed".into()))?;
    Ok(nonce)
}

fn parse_ciphertext(value: &str) -> ProviderResult<([u8; NONCE_LEN], Vec<u8>)> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 || parts[0] != CIPHER_VERSION {
        return Err(ProviderError::Secret("provider key ciphertext format is invalid".into()));
    }
    let nonce = parse_nonce(parts[1])?;
    let ciphertext = hex::decode(parts[2]).map_err(|_| ProviderError::Secret("provider key ciphertext is not hex".into()))?;
    Ok((nonce, ciphertext))
}

fn parse_nonce(value: &str) -> ProviderResult<[u8; NONCE_LEN]> {
    let bytes = hex::decode(value).map_err(|_| ProviderError::Secret("provider key nonce is not hex".into()))?;
    bytes
        .try_into()
        .map_err(|_| ProviderError::Secret("provider key nonce length is invalid".into()))
}

#[cfg(test)]
mod tests {
    use crate::{
        application::SecretCipher,
        infra::secret_cipher::{CIPHER_VERSION, NONCE_LEN, ProviderKeyCipher},
    };

    #[test]
    fn encrypt_provider_key_returns_versioned_ciphertext() {
        let cipher = ProviderKeyCipher::new("provider-secret".into()).unwrap();

        let encrypted = cipher.encrypt_provider_key("sk-provider-key").unwrap();

        let parts = encrypted.split(':').collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], CIPHER_VERSION);
        assert_eq!(hex::decode(parts[1]).unwrap().len(), NONCE_LEN);
        assert!(hex::decode(parts[2]).unwrap().len() > "sk-provider-key".len());
        assert_ne!(encrypted, "sk-provider-key");
    }

    #[test]
    fn encrypt_provider_key_uses_unique_nonce() {
        let cipher = ProviderKeyCipher::new("provider-secret".into()).unwrap();

        let first = cipher.encrypt_provider_key("sk-provider-key").unwrap();
        let second = cipher.encrypt_provider_key("sk-provider-key").unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn decrypt_provider_key_returns_original_plaintext() {
        let cipher = ProviderKeyCipher::new("provider-secret".into()).unwrap();
        let encrypted = cipher.encrypt_provider_key("sk-provider-key").unwrap();

        let decrypted = cipher.decrypt_provider_key(&encrypted).unwrap();

        assert_eq!(decrypted, "sk-provider-key");
    }

    #[test]
    fn decrypt_provider_key_rejects_invalid_ciphertext() {
        let cipher = ProviderKeyCipher::new("provider-secret".into()).unwrap();

        let result = cipher.decrypt_provider_key("sk-provider-key");

        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_blank_secret() {
        let result = ProviderKeyCipher::new("  ".into());

        assert!(result.is_err());
    }
}
