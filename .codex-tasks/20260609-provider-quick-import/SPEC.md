# Provider Quick Import

## Goal

Implement the newapi quick import flow for provider management.

## Boundary

- Backend preview and commit APIs for creating a new provider from newapi tokens.
- Frontend provider management modal for the two-step import flow.
- RBAC and admin i18n seed updates.
- Automated validation for Rust and frontend checks where feasible.

## Constraints

- No silent fallbacks or mock success paths.
- Commit must be transactional.
- `newapi` token keys returned without `sk-` must be normalized before use and storage.
- Unknown or unmapped selected models must block commit.
