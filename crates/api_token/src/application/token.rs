use rand::{Rng, distributions::Alphanumeric};
use rand_core::OsRng;
use sha2::{Digest, Sha256};

const TOKEN_RANDOM_LEN: usize = 32;
const TOKEN_PREFIX: &str = "sk-";
const DISPLAY_PREFIX_LEN: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedToken {
    pub value: String,
    pub hash: String,
    pub prefix: String,
}

pub fn generate_token() -> GeneratedToken {
    let random_part: String = OsRng.sample_iter(&Alphanumeric).take(TOKEN_RANDOM_LEN).map(char::from).collect();
    let value = format!("{TOKEN_PREFIX}{random_part}");
    let hash = hash_token(&value);
    let prefix = value.chars().take(DISPLAY_PREFIX_LEN).collect();
    GeneratedToken { value, hash, prefix }
}

pub fn hash_token(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(digest)
}

#[cfg(test)]
mod tests {
    use super::{generate_token, hash_token};

    #[test]
    fn generate_token_uses_aether_key_format() {
        let token = generate_token();

        assert_eq!(token.value.len(), 35);
        assert!(token.value.starts_with("sk-"));
        assert!(token.value["sk-".len()..].chars().all(|value| value.is_ascii_alphanumeric()));
        assert_eq!(token.prefix, token.value.chars().take(10).collect::<String>());
        assert_eq!(token.hash, hash_token(&token.value));
    }

    #[test]
    fn hash_token_matches_aether_sha256_hex() {
        assert_eq!(
            hash_token("sk-PCr5oXZNKb9HcyzYqTIMvr8zXsIBK3WS"),
            "2c44b1b522f888150a5128bce370918da330ec82122c8cb26f382f2961c7cb61"
        );
    }
}
