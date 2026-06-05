# Hook Release Package

This package contains the Hook backend binary with embedded frontend assets.

## Contents

- `bin/hook_backend`: Hook API server and embedded web UI.
- `bin/generate_password_hash`: helper for generating the initial admin password hash.
- `config/config.example.yaml`: binary deployment configuration template.
- `LICENSE`: MIT license.

## Runtime Requirements

Binary packages do not start PostgreSQL or Redis. Provide both services before running Hook.

Docker Compose Source Build remains the recommended first deployment path when you want Hook, PostgreSQL, and Redis managed together.

## Install From GitHub Release

```bash
curl -fsSL https://github.com/zzispp/Hook/releases/latest/download/install.sh | sudo bash
```

To install a specific version:

```bash
curl -fsSL https://github.com/zzispp/Hook/releases/download/v0.1.0/install.sh | sudo bash -s -- --version v0.1.0
```

The installer downloads the platform package, verifies it against `SHA256SUMS`, installs it under `/opt/hook/releases/<version>`, updates `/opt/hook/current`, and creates `/etc/hook/config.yaml` when it does not already exist.

## Configure

Edit `/etc/hook/config.yaml` after installation.

Generate an admin password hash:

```bash
/opt/hook/current/bin/generate_password_hash "your-password"
```

Set these values before starting Hook:

- `database.url`
- `redis.url`
- `jwt.secret`
- `security.provider_key_secret`
- `admin.username`
- `admin.email`
- `admin.password_hash`

## Run

```bash
/opt/hook/current/bin/hook_backend --config /etc/hook/config.yaml migration up
/opt/hook/current/bin/hook_backend --config /etc/hook/config.yaml
```
