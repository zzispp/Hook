# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Add dynamic system contact methods to admin system settings.
- Persist contacts in the existing global `system_settings` record.
- Expose contacts through `/api/site-info`.
- Support built-in contact types plus custom type labels, Iconify icons, text/link values, and QR code data URL or HTTP(S) URL.

## Non-Goals

- Do not add a new contact CRUD route or attachment service.
- Do not render contacts on a specific public page in this task.
- Do not add fallback/mock behavior for invalid contact data.

## Constraints

- Follow existing baseline-only migration pattern.
- Keep admin copy backend-seeded through `admin` i18n resources.
- Surface validation failures explicitly.
- Respect current Iconify offline registration pattern.

## Environment

- **Project root**: `/Users/bubu/ZwjProjects/Hook`
- **Language/runtime**: Rust workspace + Next.js/TypeScript
- **Package manager**: `pnpm`
- **Test framework**: Rust `cargo test`; frontend lint/build
- **Build command**: `pnpm build:frontend`

## Deliverables

- Rust types, storage, migration baseline, seed, validation, and tests for `contact_methods`.
- Frontend system settings contacts tab with dynamic row editing and QR code upload/URL support.
- Registered default contact icons.
- CN/EN admin i18n seed keys.

## Done-When

- [ ] Contacts save and reload through existing system settings API.
- [ ] `/api/site-info` includes contacts.
- [ ] Backend rejects invalid contact payloads with explicit errors.
- [ ] Frontend lint/build or targeted equivalent passes.

## Final Validation Command

```bash
timeout 60 cargo test -p setting -p storage -p types && pnpm lint:frontend && pnpm build:frontend
```

