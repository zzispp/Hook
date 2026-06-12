# Task Specification

## Task Shape

- **Shape**: single-full

## Goals

- Align Hook scheduling with Aether-style required capability hard filtering.
- Route explicit image generation requests only to candidates whose global model and provider key declare `image_generation`.
- Expose provider key capabilities in admin key create/update payloads.

## Non-Goals

- No production server mutation or deploy.
- No fallback from incapable image keys to generic chat or image endpoints.
- No automatic inference from `api_formats` to capabilities.

## Constraints

- Existing dirty worktree changes are preserved.
- Backend tests run with a 60 second timeout.
- Admin i18n remains backend-seeded.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust backend, Next.js frontend
- **Package manager**: pnpm
- **Test framework**: cargo tests and frontend lint/build

## Deliverables

- Provider API key `capabilities` field in storage/API/cache.
- Request-level `required_capability` for image generation/edit paths.
- Candidate filtering by global model and key capabilities.
- Targeted Rust tests for supported and unsupported capability candidates.

## Done-When

- [ ] Capability fields compile through backend and frontend types.
- [ ] Image generation candidate selection excludes missing model capability.
- [ ] Image generation candidate selection excludes missing key capability.
- [ ] Validation commands complete or failures are reported.

## Final Validation Command

```bash
cargo fmt --check && timeout 60 cargo test -p hook_backend capability && timeout 60 cargo check -p hook_backend
```
