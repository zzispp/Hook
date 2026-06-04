# Landing CSS Inline Migration

## Goal

Convert the landing page CSS touched between `c4067752d4f8987b2bec3958041cc9aff923d37b` and `HEAD` to the project's existing frontend styling approach, without standalone React Bits CSS files.

## Scope

- Inspect CSS files and imports touched in the commit range.
- Move landing styles into TypeScript/TSX style ownership.
- Remove standalone React Bits CSS files and global layout import.
- Validate frontend lint/build behavior where feasible.

## Constraints

- Preserve current landing page behavior and class contracts.
- Do not introduce fallback behavior or mock paths.
- Keep unrelated backend and existing commit-range changes untouched.
