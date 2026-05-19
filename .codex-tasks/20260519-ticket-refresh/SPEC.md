# Ticket Refresh

## Goal

Add an explicit refresh action to the ticket workspace so users/admins can reload ticket data on demand.

## Boundary

- Frontend ticket workspace only.
- No polling interval or silent fallback behavior unless explicitly approved later.
- Preserve existing SWR data flow and admin/user ticket endpoints.
