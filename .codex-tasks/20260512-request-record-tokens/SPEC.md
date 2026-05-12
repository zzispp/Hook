# Request record token display

## Goal

Persist token usage from upstream responses and show it in admin request records.

## Scope

- Parse token usage for OpenAI, Claude, Gemini, and converted responses.
- Store prompt, completion, and total tokens on request candidate records.
- Aggregate token fields into request record list/detail responses.
- Keep failures visible when usage is absent or unparseable.
