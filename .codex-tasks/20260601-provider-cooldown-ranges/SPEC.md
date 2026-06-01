# Provider Cooldown Status Ranges

## Goal

Support provider cooldown policy rules that match either one HTTP status code or an inclusive status-code range such as `502-504`.

## Scope

- Replace rule `status_code` with `status_code_start` and `status_code_end`.
- Validate non-overlapping ranges from 100 through 599.
- Count failures per matching range rule, not per actual status code.
- Keep recorded cooldown rows storing the actual triggering status code.

