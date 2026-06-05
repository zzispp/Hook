# Progress

## 2026-06-05

- Confirmed there is no existing `.github` workflow.
- Confirmed Rust checks are exposed through `justfile` and `package.json` uses `cargo check -p hook_backend`.
- Confirmed frontend requires Node >= 22.12 and pnpm 10.33.4.
- Confirmed Docker deployment has a source-build Dockerfile and Compose config, so CI should build locally but not publish an image.
- Added `.github/workflows/ci.yml` with Rust, Frontend, and Docker jobs.
- Rust job uses `cargo check -p hook_backend`, workspace clippy, full test compilation, and a 60-second backend crate test run.
- Frontend job installs pnpm 10.33.4 on Node 24, then runs lint and embedded static build.
- Docker job validates deployment scripts, validates Compose config with CI env values, and builds the Docker image without pushing.
- Local validation exposed existing clippy/test drift; fixed stale affiliate response fixture fields, direct bool assertions, unit struct `Default` construction in proxy tests, an unused lifetime, and a long test support return type.
- Validated locally: `cargo fmt --all --check`, `cargo check -p hook_backend`, `just lint`, `cargo test --workspace --all-targets --no-run`, `timeout 60s cargo test -p hook_backend --all-targets`, `pnpm lint:frontend`, `pnpm build:frontend:embedded`, workflow YAML parsing, deployment script syntax, Docker Compose config, and `docker build -t hook:ci-local .`.
