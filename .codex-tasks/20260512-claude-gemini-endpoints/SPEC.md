# Claude and Gemini client endpoints

## Goal

Expose Claude and Gemini client-compatible routes through the existing provider routing system.

## Scope

- Add `/v1/messages` for Claude Messages clients.
- Add `/v1beta/models/{model}:{action}` for Gemini GenerateContent clients.
- Preserve existing OpenAI `/v1` routes.
- Use request path as the client format and provider endpoint format as upstream format.

