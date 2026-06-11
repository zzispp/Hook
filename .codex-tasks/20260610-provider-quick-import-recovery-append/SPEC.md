# Quick Import Recovery, Model Associations, And Append Import

## Goal
Implement quick import provider append import, key recovery/relink, explicit model association management, hard-anomaly enable blocking, and one-time model candidate notifications.

## Boundary
- Reuse the existing quick import source metadata for append/recovery flows.
- Keep manual providers out of scope.
- Do not add frontend locale JSON; update backend admin i18n seed only.
- Preserve existing user changes in the dirty worktree.
