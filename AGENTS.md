# Repository Guidelines

## Project Structure & Module Organization

This is a Rust and pnpm monorepo. Rust workspace members live in `apps/hook_backend` and `crates/*`; shared domain modules are split into `crates/config`, `crates/constants`, `crates/storage`, `crates/types`, and `crates/user`. Frontend packages are declared in `pnpm-workspace.yaml`: `apps/hook_frontend` contains the Next.js UI, while `apps/hook_mock_api` provides mock API routes and bundled assets. Static assets belong under each app's `public/` directory, and environment-style YAML configuration is stored in `config/`.

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

## Security & Configuration Tips

Do not commit secrets or local credentials. Keep runtime configuration in `config/` or environment variables, and document any new required setting in the relevant app or crate README.
