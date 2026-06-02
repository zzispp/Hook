# Model Status Dispatch Completion

## Goal

Make model status dispatch clear all currently due checks in one scheduled run, while each probe still performs a real model request through the existing proxy candidate retry/degrade policy.

## Constraints

- Do not introduce mock success, silent fallback, or hidden skip behavior.
- Preserve provider key probe minimum interval as an explicit pacing rule.
- Bound provider key probe slot waiting with an explicit timeout so dispatch cannot block forever.
- Keep request failures visible through existing proxy audit records.

## Validation

- `timeout 60 cargo test -p model_status`
- `timeout 60 cargo test -p backend`
