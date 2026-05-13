# Trace Candidate Compaction

## Goal

Reduce request trace dot explosion caused by flattening endpoint, key, and retry slots into visible attempt records.

## Scope

- Keep request scheduling semantics explicit and observable.
- Avoid pre-creating retry records that were not attempted.
- Mark records left unused after success with a concrete status.
- Improve frontend grouping so dots represent meaningful attempts rather than raw flat candidate rows.
- Validate backend and frontend checks where feasible.

## Non-Goals

- Do not add silent caps or hidden fallback behavior.
- Do not mock upstream execution.
- Do not rewrite provider/channel architecture beyond the requested trace and candidate pressure issue.
