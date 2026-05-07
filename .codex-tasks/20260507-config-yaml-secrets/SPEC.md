# Config YAML Secrets

## Goal

Move secret-like runtime values that were still resolved through environment variable indirection into `config/config.yaml` backed settings.

## Scope

- Replace `jwt.secret_env` with `jwt.secret`.
- Replace `admin.password_hash_env` with `admin.password_hash`.
- Replace `redis.url_env` with `redis.url`.
- Keep configuration failures explicit for blank values.
- Validate with Rust tests/checks and backend startup.
