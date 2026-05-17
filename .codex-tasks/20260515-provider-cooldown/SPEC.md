# Provider Cooldown

Implement global provider cooldown policy and runtime cooldown state.

Requirements:
- Add global provider cooldown policy to system settings.
- Record provider-level cooldowns triggered by configured HTTP status codes within a fixed time window.
- Filter cooled providers out of proxy scheduling without confusing cooldown with provider manual active state.
- Expose admin cooldown list and release API.
- Add provider management tabs, policy dialog, cooldown table, and admin i18n seed keys.
- Keep failures explicit; do not add silent fallback paths.

Validation:
- `just test`
- `pnpm lint:frontend`
- `pnpm build:frontend`
