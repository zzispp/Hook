# Hero Gateway Copy

## Goal

Adjust the React Bits landing Hero section so it describes Hook as an AI model unified gateway and shows OpenAI-compatible API usage examples.

## Scope

- Update `apps/hook_frontend/src/react-bits/components/landingnew/Hero/Hero.tsx`.
- Update related Hero CSS only where required by the new code panel behavior.
- Keep the existing visual background and layout intact.

## Acceptance Criteria

- Hero copy is specific to Hook as an AI gateway/control plane.
- Right-side code window switches between cURL, Node.js, and Python examples.
- Examples use the current host as the API base URL with `/v1`.
- Frontend lint passes or any failure is recorded with concrete output.
