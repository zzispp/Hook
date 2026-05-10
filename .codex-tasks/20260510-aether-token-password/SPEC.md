# Aether Token And Password Compatibility

## Goal

- Match Hook API token raw format with Aether generated API keys.
- Match Hook password hashing and verification with Aether so SQL migration can preserve password_hash values.
- Seed/config admin as username admin, email admin@example.com, password 123456 hashed by Aether-compatible algorithm.
