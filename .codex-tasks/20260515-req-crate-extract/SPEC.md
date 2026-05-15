# req crate extract

## Goal
Extract the reusable HTTP request utilities from the monorepo into a dedicated Rust crate under `crates/req`, following the same idea as the `gem_client` module in `corn_wallet_ios/core`.

## Scope
- Add a new workspace crate for request helpers.
- Move reusable HTTP client helpers into that crate.
- Update in-repo consumers to use the new crate where the abstraction fits.
- Keep protocol-specific request shaping inside the owning feature crates.

## Assumptions
- The extraction should focus on shared HTTP/client plumbing, not on LLM proxy business logic.
- The new crate should fit the existing workspace layout and Rust style.

## Validation
- Workspace builds successfully.
- Targeted tests for the new crate pass.
- Existing crates that adopt the new helper continue to compile.
