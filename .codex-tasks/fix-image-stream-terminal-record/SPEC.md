# Fix Image Stream Upstream Semantics

## Goal

OpenAI image generation stream requests must keep true streaming semantics end to end: upstream image requests preserve `stream:true`, and successful EOF records stay terminal instead of being overwritten back to `streaming`.

## Scope

- Reproduce why `openai_image` stream records stay `streaming` after EOF and why large image stream requests can be timed as first-byte failures.
- Preserve `stream:true` for OpenAI image generation/edit upstream bodies.
- Fix terminal record persistence so successful EOF updates status, billing, latency, and finished state.
- Add focused regression coverage for OpenAI image stream upstream request rewrite and EOF terminal state.
- Run targeted backend tests and formatting.
