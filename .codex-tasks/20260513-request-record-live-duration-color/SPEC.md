# Request Record Live Duration Color

## Goal

Change the live-updating duration text on the request records page so the first-byte and total-duration values render in red while the request is still in progress, then return to the existing default color once the request completes.

## Scope

- Request records duration text rendering.
- Frontend validation for the admin request records page.

## Acceptance

- Live first-byte duration text is red while the request is pending or streaming.
- Live total-duration text is red while the request is pending or streaming.
- Completed durations render with the existing non-red styling.
- Frontend lint/build checks pass.
