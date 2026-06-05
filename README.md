# Hook

<p align="center">
  <img src="apps/hook_frontend/public/logo/logo.svg" width="240" alt="Hook Logo">
</p>

<p align="center">
  <strong>AI API Gateway and Operations Platform</strong>
</p>

<p align="center">
  English • <a href="README_CN.md">中文</a>
</p>

---

## Overview

Hook is a Rust and pnpm monorepo for an AI API gateway, user console, and operations admin panel. The backend uses Axum, SeaORM, Redis, and PostgreSQL for proxying, authentication, scheduling, billing, monitoring, and admin APIs. The frontend uses Next.js, React, MUI, and TypeScript.

Users call Hook with Hook-issued tokens through `/v1` or `/v1beta`. Hook routes requests by model, provider, group, wallet balance, and permission policy, then records usage, billing, and runtime status.

## Features

- **Unified AI proxy**: OpenAI-style `/v1` and Gemini-style `/v1beta` routes for chat, Responses, Claude Messages, images, embeddings, rerank, audio, moderations, and Realtime.
- **Provider and model management**: Global models, providers, endpoints, upstream API keys, model bindings, model costs, cooldown release, and model connectivity tests.
- **Tokens and permissions**: User tokens, admin tokens, RBAC roles, menus, API permissions, navigation permissions, and system token policy.
- **Wallet and billing**: User wallets, balances, transactions, daily model usage, admin adjustments, admin recharge, billing groups, user groups, and price groups.
- **Recharge, card codes, and affiliates**: Recharge packages, payment channels, callback records, card-code generation and redemption, affiliate relations, commissions, and report exports.
- **Request records and cost analytics**: Client and upstream request records, active requests, usage history, user stats, cost forecast, savings, and aggregation stats.
- **Model status and operations monitoring**: Model status checks, scheduled tasks, cache affinity monitoring, system performance monitoring, and health checks.
- **Account and operations admin**: Sign-up, sign-in, token refresh, OAuth, wallet sign-in, password reset, profile, announcements, tickets, notifications, and site settings.
- **Backend-driven admin i18n**: Admin copy is served from `translation_languages` and `translation_entries` through `/api/i18n/resources`.

## Repository Layout

```text
.
├── apps/
│   ├── hook_backend/      # Axum backend, LLM proxy, migrations, monitoring, scheduling
│   └── hook_frontend/     # Next.js user console and admin panel
├── crates/                # Shared Rust domain modules
├── config/config.yaml     # Default local config
├── Dockerfile             # Multi-stage image with embedded frontend assets
├── docker-compose.yml     # PostgreSQL, Redis, and Hook source-build deployment
├── deploy.sh              # One-command Docker Compose source-build deployment
├── update.sh              # One-command Docker Compose source-build update
├── package.json           # pnpm workspace scripts
├── Cargo.toml             # Rust workspace
└── justfile               # Rust build, check, test, and migration commands
```

Brand assets:

- `apps/hook_frontend/public/logo/logo.svg`: full logo with text.
- `apps/hook_frontend/public/logo/logo-icon.svg`: icon logo.

## Deployment

### Docker Compose Source Build

Docker Compose is the recommended deployment path for the current stable version. It builds Hook from source, compiles the embedded frontend, starts PostgreSQL and Redis, runs `migration up`, then serves the frontend and API from `http://127.0.0.1:5555`.

```bash
git clone https://github.com/zzispp/Hook.git
cd Hook
./deploy.sh
```

`./deploy.sh` creates `.env` on first run, asks for the administrator username, email, and password, then starts Docker Compose. PostgreSQL password, JWT secret, and provider key encryption secret are generated automatically. PostgreSQL and Redis runtime data is stored in the `hook-postgres` and `hook-redis` Docker named volumes.

Useful commands:

```bash
docker compose logs -f hook
docker compose ps
docker compose down
```

`docker compose down` stops containers without deleting named volumes. Do not run `docker compose down -v` unless the deployment data should be deleted.

### One-Command Update

After Docker Compose deployment, run this command in the deployment directory:

```bash
./update.sh
```

`update.sh` runs `git pull --ff-only`, pulls the PostgreSQL and Redis base images, rebuilds the Hook image from the current source tree, and recreates the containers. It does not delete Docker named volumes.

### Source Build Without Docker

Source build deployment uses the same embedded frontend path without Docker. PostgreSQL and Redis must be running, and `config.yaml` must point to them before the migration command runs.

```bash
pnpm install
cp config/config.yaml config.yaml
scripts/generate-password-hash.sh "your-password"
# Update config.yaml with the printed password hash, database, Redis, and secret values.
cargo run -p hook_backend -- migration up
pnpm build:frontend:embedded
cargo run -p hook_backend
```

## Local Development

Requirements: Rust edition 2024 toolchain, Node.js `>=22.12.0`, pnpm `10.33.4`, PostgreSQL, and Redis. The default config points to `localhost:5433` and `localhost:6380`.

Install dependencies, prepare config, initialize the database, generate embedded frontend assets, and run both dev servers:

```bash
pnpm install
cp config/config.yaml config.yaml
scripts/generate-password-hash.sh "your-password"
cargo run -p hook_backend -- migration up
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm build:frontend:embedded
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm dev
```

Development URLs: backend `http://127.0.0.1:5555`, frontend `http://127.0.0.1:8082`.

## Build, Config, and Database

```bash
pnpm build:frontend
pnpm build:backend
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm build:backend:embedded
NEXT_PUBLIC_SERVER_URL=http://127.0.0.1:5555 pnpm start:embedded
```

Important config keys include `server.*`, `database.*`, `redis.*`, `jwt.secret`, `security.provider_key_secret`, `admin.*`, `auth.*`, `tracing.log_level`, `NEXT_PUBLIC_SERVER_URL`, and `HOOK_BACKEND_URL`.

Migration commands:

```bash
cargo run -p hook_backend -- migration up       # Apply baseline without clearing complete existing tables
cargo run -p hook_backend -- migration status   # Show baseline table status
cargo run -p hook_backend -- migration down     # Drop baseline tables and marker
cargo run -p hook_backend -- migration fresh    # Drop and recreate baseline
cargo run -p hook_backend -- migration refresh  # Drop and recreate baseline
cargo run -p hook_backend -- migration reset    # Drop baseline tables and marker
```

## API Entrypoints

- `GET /health`: health check.
- `/api/*`: console, admin, auth, wallet, billing, provider, model, and monitoring APIs.
- `/v1/*`: OpenAI, Claude, and Jina compatible proxy.
- `/v1beta/*`: Gemini compatible proxy.

Common proxy routes include `/v1/models`, `/v1/chat/completions`, `/v1/responses`, `/v1/messages`, `/v1/images/generations`, `/v1/embeddings`, `/v1/rerank`, `/v1/realtime`, `/v1beta/models/{model}:generateContent`, and `/v1beta/models/{model}/embedContent`.

## Validation

```bash
just check
just lint
just test
pnpm lint:frontend
pnpm build:frontend
```

## Acknowledgments

Hook references and is inspired by the following open-source projects:

- [looplj/axonhub](https://github.com/looplj/axonhub)
- [fawney19/Aether](https://github.com/fawney19/Aether)
- [QuantumNous/new-api](https://github.com/QuantumNous/new-api)

## License

Hook is released under the [MIT License](./LICENSE).
