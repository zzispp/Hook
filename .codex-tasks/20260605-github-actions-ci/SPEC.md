# GitHub Actions CI

## Goal

Add a GitHub Actions workflow that matches Hook's Rust + pnpm monorepo and validates pull requests with the same commands documented for local development.

## Scope

- Run Rust formatting, backend check, clippy, and tests.
- Run frontend dependency install, lint, and embedded static build.
- Validate Docker deployment files with shell syntax, Compose config, and Docker image build.
- Avoid publishing artifacts or images.

## Validation

- Workflow YAML parses.
- Local commands referenced by the workflow are valid for the repository.
- Docker Compose config resolves with a generated CI env file.
