# Ops Pagination And Notification Freshness

## Goal

- Admin ticket list should prioritize unread or unfinished work.
- Admin announcement and ticket management pages should expose pagination controls instead of fixed first-page fetching.
- Notification drawer should refresh without full page reload using the recommended low-complexity strategy.

## Acceptance

- Ticket storage ordering prioritizes admin actionable/unresolved tickets, then latest activity.
- Frontend admin pages pass table page and rowsPerPage to backend APIs and render pagination controls.
- Notification SWR config refreshes on focus/reconnect, polls while the document is visible, and drawer open/tab switch explicitly refreshes data.
