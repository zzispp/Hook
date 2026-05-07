# Backend Schema Startup

## Goal

Fix backend startup failure when the PostgreSQL table already exists.

## Finding

Toasty `push_schema()` is documented as a prototyping/testing schema push. It directly issues create-table statements and is not tracked as an idempotent migration for existing PostgreSQL tables. Calling it unconditionally at service startup causes `relation "user_records" already exists` after the first successful initialization.

## Decision

Do not call `push_schema()` unconditionally during normal backend startup. Make schema push an explicit database configuration flag so startup does not mutate schema unless requested.
