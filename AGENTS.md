# Repository Guidelines

## Project Structure & Module Organization

This is a Rust and pnpm monorepo. Rust workspace members live in `apps/hook_backend` and `crates/*`; shared domain modules are split into `crates/config`, `crates/constants`, `crates/storage`, `crates/types`, and `crates/user`. Frontend packages are declared in `pnpm-workspace.yaml`: `apps/hook_frontend` contains the Next.js UI. Static assets belong under each app's `public/` directory, and environment-style YAML configuration is stored in `config/`.

## Build, Test, and Development Commands

- `pnpm install`: install JavaScript workspace dependencies.
- `pnpm dev:frontend`: run the frontend on port `8082`.
- `pnpm dev:mock-api`: run the mock API on port `7272`.
- `pnpm build:frontend` / `pnpm build:mock-api`: build the Next.js apps.
- `pnpm lint:frontend` / `pnpm lint:mock-api`: run ESLint for TypeScript and React code.
- `just check`: run `cargo check` for the Rust workspace.
- `just build`: build all Rust crates.
- `just test`: run Rust tests with the repository's 60-second timeout wrapper.

## Coding Style & Naming Conventions

TypeScript uses Prettier with 2-space indentation, semicolons, single quotes, `printWidth: 100`, and trailing commas where valid in ES5. ESLint enforces React hooks rules, sorted imports, unused-import detection, and type-import consistency. Prefer `src/` absolute imports where existing patterns use them. Rust uses edition 2024 and `rustfmt.toml` with `max_width = 160`; keep crate names lowercase and module names snake_case.

## I18n Guidelines

Admin UI copy is backend-controlled. Do not add or restore frontend `admin.json` locale files under `apps/hook_frontend/src/locales/langs/**`; admin translations must be loaded through `/api/i18n/resources` and rendered with `t()` from the `admin` namespace.

Backend translation data lives in `translation_languages` and `translation_entries`. Baseline seed JSON belongs under `apps/hook_backend/src/migration/defaults/i18n/`, currently `admin.cn.json` and `admin.en.json`; update these seed files when adding admin UI keys that must exist in a fresh development baseline.

Use stable translation keys for UI copy: table headers, form labels, placeholders, button text, validation text, empty states, dialogs, admin page chrome, and Dashboard navigation labels. Keep key names descriptive and grouped by feature, matching the backend resource structure.

Dashboard navigation labels are translated by database code. Menu sections use `nav.<menu_section.code>` and menu items use `nav.<menu_item.code>`, for example `nav.admin_wallets`. Menu management tables and forms still display the raw database `title` and `subheader` values because those fields are administrator-facing configuration values.

Do not translate other dynamic business/configuration values with frontend locale keys. API names, role names, model names, group names, usernames, and other database-owned values must be displayed as the raw backend value so administrators can control them in the database or admin pages.

Frontend admin pages should assume the backend resource is required. Do not add silent fallback copy, mock translation resources, or compatibility imports from removed locale JSON files. If translations fail to load, surface the real error.

When changing i18n behavior, validate both sides: run backend checks for the i18n API or seed path, and run frontend TypeScript/lint checks for pages that consume `t()`.

## Testing Guidelines

No JavaScript test runner is configured yet; rely on linting and Next.js builds for UI validation. Rust tests should be colocated with the crate they validate using normal `#[cfg(test)]` modules or integration tests when a public API boundary is required. Run `just test` before submitting Rust changes, and keep tests deterministic and under the configured timeout.

## Commit & Pull Request Guidelines

The current history uses Conventional Commit style, for example `chore: init monorepo`. Continue with concise messages such as `feat: add user profile route` or `fix: validate config path`. Pull requests should describe the change, list validation commands run, link related issues, and include screenshots or screen recordings for visible frontend changes.

## Development & Release Workflow

Use feature branches for normal development. Write code on a branch, open a pull request, wait for CI, then merge into `main`. Direct pushes to `main` should be reserved for small maintainer-only changes where review is not needed.

Commit messages drive changelog and version calculation through Release Please, so keep them in Conventional Commit format:

- `fix: ...` for patch releases.
- `feat: ...` for minor releases.
- `feat!: ...`, `fix!: ...`, or a `BREAKING CHANGE:` footer for major releases.
- `docs: ...`, `chore: ...`, and CI-only changes are allowed, but they usually should not be relied on to create a product release by themselves.

Every push to `main` runs CI and builds the GHCR development Docker images:

- `ghcr.io/zzispp/hook:edge`
- `ghcr.io/zzispp/hook:nightly`

Every push to `main` also runs Release Please. Release Please creates or updates a release pull request that contains the generated `CHANGELOG.md`, version updates, and `.release-please-manifest.json` changes. Do not manually edit release changelog or version files for a normal stable release; review and merge the Release Please pull request instead.

Stable release flow:

1. Merge feature pull requests into `main`.
2. Let Release Please create or update the release pull request.
3. Review and merge the Release Please pull request.
4. The merge creates the stable tag, for example `v1.0.1`.
5. The stable tag triggers GitHub Release packaging, checksum generation, Docker image publishing, and GitHub Pages deployment.

Stable release Docker tags include:

- `ghcr.io/zzispp/hook:vX.Y.Z`
- `ghcr.io/zzispp/hook:X.Y.Z`
- `ghcr.io/zzispp/hook:latest`

Prerelease flow is manual. Run the `Prerelease Tag` workflow from GitHub Actions, set `base_version` such as `1.0.1`, and choose `beta` or `rc`. The workflow creates the next available tag, for example `v1.0.1-beta.1` or `v1.0.1-rc.1`. Prerelease tags trigger GitHub prerelease packaging and Docker image publishing, but they do not deploy GitHub Pages.

## Security & Configuration Tips

Do not commit secrets or local credentials. Keep runtime configuration in `config/` or environment variables, and document any new required setting in the relevant app or crate README.
