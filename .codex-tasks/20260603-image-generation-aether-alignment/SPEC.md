# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Align Hook image-generation routing behavior with Aether for OpenAI Chat/Responses requests that explicitly force the `image_generation` tool.
- Route explicit image-generation intent to image-capable upstream handling instead of silently deleting tools or sending the request to normal chat providers.

## Non-Goals

- Do not remove `image_generation` from request bodies.
- Do not treat a mere `tools: [{"type":"image_generation"}]` declaration as image-generation intent.
- Do not change unrelated tool-choice behavior.

## Constraints

- Rust workspace in `/Users/bubu/ZwjProjects/Hook`.
- Backend tests use `cargo test`; focused backend unit tests must run with `timeout 60`.
- Follow Debug-First: unsupported image capabilities should surface explicit errors instead of silent degradation.

## Deliverables

- Intent detection equivalent to Aether: only `tool_choice.type = image_generation` forces image generation.
- Candidate routing/body conversion support for that intent.
- Focused tests proving explicit intent routes to image handling while tool declaration alone does not.

## Done-When

- [ ] Explicit OpenAI Responses image-generation tool choice routes to an image-capable provider path.
- [ ] Explicit OpenAI Chat image-generation tool choice routes to an image-capable provider path.
- [ ] Tools declaration without matching `tool_choice` remains normal chat/Responses routing.
- [ ] No request-body mutation removes `image_generation`.

## Final Validation Command

```bash
timeout 60 cargo test -p hook_backend image_generation
```
