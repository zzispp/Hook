use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

const TOKEN_BYTES: usize = 32;
const TOKEN_PREFIX: &str = "hk_";
const DISPLAY_PREFIX_LEN: usize = 14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedToken {
    pub value: String,
    pub hash: String,
    pub prefix: String,
}

pub fn generate_token() -> GeneratedToken {
    let mut bytes = [0_u8; TOKEN_BYTES];
    OsRng.fill_bytes(&mut bytes);
    let value = format!("{TOKEN_PREFIX}{}", hex::encode(bytes));
    let hash = hash_token(&value);
    let prefix = value.chars().take(DISPLAY_PREFIX_LEN).collect();
    GeneratedToken { value, hash, prefix }
}

pub fn hash_token(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)
}
