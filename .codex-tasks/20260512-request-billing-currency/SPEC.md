# Request Billing And Currency

## Goal
- Compute request cost from model pricing and billing group multiplier instead of showing all zero.
- Add a system currency setting with USD as default and CNY option.
- Display monetary values in the configured currency across admin request record and pricing surfaces touched by this feature.
- Add a dedicated exchange-rate refresh task that writes USD/CNY rates to Redis every 5 minutes.

## External Source Decision
- Default public source: Frankfurter latest endpoint, no API key required.
- The source exposes latest working-day reference rates, not tick-level real-time FX.
- Scheduler still wakes every 5 minutes and refreshes Redis so UI/billing conversion reads a local cache.

## Constraints
- No fake billing values.
- Billing group multiplier must be applied after model token/per-request pricing.
- Failures must be visible in logs; do not silently return invented rates.
