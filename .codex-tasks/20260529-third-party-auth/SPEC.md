# Task Specification

> Scope anchor for the task. Update only when goals or constraints change, and log the reason in PROGRESS.md.

## Task Shape
<!-- single-compact | single-full | epic | batch -->

- **Shape**: `single-full`

## Goals
<!-- What are we building? Be specific and concrete. -->

- Implement third-party quick login and account binding for GitHub, Google, EVM wallet, and Solana wallet while keeping local users as the source of truth.
- Add current-user profile page with password status, email-OTP password changes, and Provider unlinking.
- Expose Provider binding status in admin user management.

## Non-Goals
<!-- What are we explicitly NOT doing? Prevents scope creep. -->

- Do not replace the existing JWT session system with Supabase, Firebase, Auth.js, Privy, or Dynamic.
- Do not make wallet login usable without an email-backed platform account.
- Do not let Provider secrets leak through public or admin read responses.

## Constraints
<!-- Tech stack, style guide, performance limits, compatibility requirements -->

- Follow existing Rust workspace, SeaORM, Axum, Next.js, MUI, SWR, and i18n patterns.
- Backend unit tests should run under the repository `just test` timeout wrapper.
- No silent fallbacks: disabled/misconfigured Providers and invalid signatures must fail explicitly.
- Admin UI translations remain backend-seeded; update backend seed JSON for admin/auth copy.

## Environment
<!-- Auto-filled by agent at init time -->

- **Project root**: `/Users/bubu/.codex/worktrees/c4fd/Hook`
- **Language/runtime**: Rust 2024 workspace, Next.js 16 / React 19 / TypeScript
- **Package manager**: pnpm
- **Test framework**: Rust unit/integration tests; frontend lint/build
- **Build command**: `just check`, `just test`, `pnpm lint:frontend`, `pnpm build:frontend`
- **Existing test count**: not counted at start

## Risk Assessment
<!-- Identify potential blockers or unknowns before starting -->

- [x] External dependencies (APIs, services) — no live third-party calls needed for tests; code should expose real errors at runtime.
- [x] Breaking changes to existing code — user password hash becomes optional and API responses expand.
- [x] Large file generation — no large generated files expected.
- [x] Long-running tests — repository uses `just test` timeout wrapper.

## Deliverables
<!-- Concrete outputs: files, features, endpoints, docs -->

- Database/model changes for identities, optional local password, and Provider settings.
- Backend auth/account/admin APIs for OAuth, wallet login, profile, password OTP, and identity unlinking.
- Frontend login buttons, callback flows, profile page, settings UI, and admin user table badges.
- Focused tests and validation commands.

## Done-When
<!-- Final acceptance criteria. The task is DONE when ALL of these pass. -->

- [ ] Existing password login still works for password-backed users.
- [ ] Third-party-created users have `password_set=false` until email-OTP password change.
- [ ] One local user can bind multiple Providers, while one Provider subject cannot bind multiple users.
- [ ] Admin user table shows Provider badges and unset-password badge.
- [ ] Profile page supports Provider listing/unlinking and email-OTP password changes.
- [ ] Validation commands pass or any failures are reported with root cause.

## Final Validation Command
<!-- Single command that validates the entire deliverable. Runs at close-out. -->

```bash
just check && just test && pnpm lint:frontend && pnpm build:frontend
```

## Demo Flow (optional)
<!-- Step-by-step instructions to demonstrate the finished product. -->

1. Configure Providers in admin settings.
2. Sign in with GitHub/Google/wallet from `/auth/sign-in`.
3. Visit `/dashboard/profile` to inspect bindings and password state.
4. Visit admin users to confirm Provider and unset-password badges.
