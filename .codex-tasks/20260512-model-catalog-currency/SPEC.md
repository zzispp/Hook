# Fix Model Catalog Currency Display

## Goal

When the backend currency setting is CNY, the public model catalog must display model prices in CNY the same way the admin model management page already does.

## Scope

- Compare the model catalog pricing flow with the admin model management pricing flow.
- Fix the source of currency selection or formatting for the model catalog only where the chain is broken.
- Validate with targeted static/build checks available in the repository.

## Constraints

- Do not add silent fallback copy, mock pricing, or compatibility behavior.
- Do not change business currency thresholds without locating read sites first.
- Keep changes scoped to the broken display path.
