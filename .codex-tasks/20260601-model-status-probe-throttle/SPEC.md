# Model Status Probe Throttle

## Goal

Prevent scheduled model status checks from bursting probes into the same provider key and causing upstream 429 responses.

## Scope

- Add an explicit model-status probe throttle marker to internal proxy requests.
- Claim a Redis-backed provider-key probe slot immediately before an upstream attempt.
- Defer throttled checks without recording a failed probe result.
- Add a scheduler setting for the provider-key minimum probe interval.
- Validate focused Rust behavior and compile checks.

## Semantics

`provider_key_min_interval_seconds` is the minimum interval between model-status probes sent to the same provider key. It applies only to model-status probes and does not replace provider RPM limits for normal traffic.
