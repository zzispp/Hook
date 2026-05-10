use bcrypt::{DEFAULT_COST, hash, verify};

use crate::application::{AppError, AppResult, PasswordHasher};

#[derive(Clone, Default)]
pub struct BcryptPasswordHasher;

impl PasswordHasher for BcryptPasswordHasher {
    fn hash(&self, password: &str) -> AppResult<String> {
        hash(password, DEFAULT_COST).map_err(password_error)
    }

    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool> {
        verify(password, password_hash).map_err(password_error)
    }
}

fn password_error(error: bcrypt::BcryptError) -> AppError {
    AppError::Infrastructure(error.to_string())
}

#[cfg(test)]
mod tests {
    use crate::{application::PasswordHasher, infra::BcryptPasswordHasher};

    #[test]
    fn hash_uses_bcrypt_format() {
        let password_hash = BcryptPasswordHasher.hash("123456").unwrap();

        assert!(password_hash.starts_with("$2"));
        assert_eq!(password_hash.len(), 60);
        assert!(BcryptPasswordHasher.verify("123456", &password_hash).unwrap());
        assert!(!BcryptPasswordHasher.verify("bad-password", &password_hash).unwrap());
    }

    #[test]
    fn verifies_configured_admin_password_hash() {
        let password_hash = "$2b$12$xQS0SfLk9OmaG69aSxN7L.hBqkBJ7i/Vty7ZVLG/nKd8nb0HV0Kaa";

        assert!(BcryptPasswordHasher.verify("12345678", password_hash).unwrap());
    }
}
