# Epic Specification

## Goal

- Implement Hook provider management and group-scoped provider proxy routing inspired by aether.
- Support Provider, Endpoint, Provider Key, provider model binding, billing group provider binding, endpoint conversion policy, request rewrite rules, scheduling, failover, and request candidate audit.

## Non-Goals

- Cost management UI.
- Model mapping UI.
- Arbitrary script execution in endpoint rewrite rules.
- Silent mock/fallback success paths.

## Constraints

- Follow the existing Rust workspace and pnpm monorepo structure.
- Keep admin i18n copy backend-controlled through seed JSON.
- Provider upstream secrets must not be hardcoded or returned in plaintext.
- Expose system/configuration failures directly; only upstream candidate failures participate in failover.
- Keep functions under project metric limits where practical.

## Risk Assessment

- Database schema changes touch existing `models.provider_id` semantics.
- Format conversion requires OpenAI, Gemini, and Claude normalizers plus stream conversion behavior.
- Proxy failover must distinguish business no-candidate errors from upstream retryable failures.
- Frontend admin changes need to align with existing MUI and backend i18n patterns.

## Child Deliverables

- Provider storage schema, types, repositories, and migrations.
- Provider admin API and route registration.
- Billing group provider binding.
- Frontend Provider management and group provider binding UI.
- API format registry and conversation conversion for OpenAI, Gemini, and Claude.
- Group-scoped proxy candidate building, scheduling, failover, and request audit.
- Validation coverage and build checks.

## Dependency Notes

- Provider storage is the foundation for admin API, group binding, frontend, and proxy.
- Proxy depends on API format conversion and storage APIs.
- Frontend depends on admin API contracts.

## Child Task Types

- `single-full`

## Done-When

- [ ] Every row in `SUBTASKS.csv` is `DONE`
- [ ] Final epic validation passes
