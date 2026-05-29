# Progress

## Recovery

- Task: Fix cache affinity deletion showing `Response data not found`.
- Shape: single-full.
- Current: Complete.
- Truth: `.codex-tasks/20260529-cache-affinity-delete/TODO.csv`.

## Notes

- `Response data not found` is thrown by `requireApiData` when `payload.data` is `null` or `undefined`.
- Backend `DELETE /admin/monitoring/cache/affinities/{...}` returns `ApiResponse<()>`, which serializes successful `data` as `null`.
- `deleteCacheAffinity` now uses the same success-envelope check as `clearCacheAffinities` instead of requiring non-null `data`.
- `pnpm lint:frontend` initially failed because `node_modules` was missing. After `pnpm install --frozen-lockfile`, the same lint command passed.
