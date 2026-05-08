# Task Specification

## Task Shape

- **Shape**: `single-full`

## Goals

- Ensure the admin model menu item receives the configured dashboard icon even when the menu already exists in the database.
- Align the model.dev picker with Aether's grouped provider list, so providers such as OpenAI and DeepSeek are shown as groups instead of a flat model list.
- Run the repository validation commands relevant to the touched backend and frontend code.

## Non-Goals

- Do not redesign the model management page.
- Do not add fake model.dev data, silent fallbacks, or compatibility-only behavior.
- Do not change unrelated RBAC/menu behavior beyond syncing default menu definitions.

## Done-When

- Existing default menu items are updated from the default definitions during RBAC initialization.
- The model.dev picker renders provider groups with expandable model lists.
- Backend and frontend checks pass, or failures are explicit with root cause.
