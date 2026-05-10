# Dashboard icons and runtime reduce error

## Goal
Fix missing dashboard nav icons for translation, system/settings, menu/API, billing/group/dashboard-related menu entries, and resolve the runtime `arr.reduce is not a function` error without adding silent fallback behavior.

## Boundary
Investigate current nav icon metadata, dashboard nav render path, and the runtime error source. Keep changes scoped to frontend nav/icon handling or baseline menu icon values if needed.
