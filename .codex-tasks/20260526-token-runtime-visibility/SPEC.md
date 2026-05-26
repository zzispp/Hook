# Runtime Token Billing Group Visibility

## Goal

Ensure an existing user token stops working when its owner moves to a user group that can no longer see the token's billing group.

## Decisions

- Use the existing LLM proxy scheduling snapshot for runtime checks.
- Do not query the database on every proxy request.
- Keep token `group_code` unchanged; requests fail when the owner group no longer has visibility.
- Apply the visibility rule to user tokens.

## Validation

- Unit tests cover user token rejection after owner group changes.
- Existing model listing and candidate selection paths keep using in-memory snapshot data.
