# Key format decoupling

Remove API format binding from provider API keys.

## Goal

- Key is only credential, priority, cache, and rate metadata.
- Endpoint decides upstream format.
- Request path decides client format.
- Conversion happens only when client format and endpoint format differ and conversion is supported.

