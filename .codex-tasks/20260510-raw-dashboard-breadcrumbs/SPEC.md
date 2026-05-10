# Raw Dashboard Breadcrumbs

## Goal

Dashboard page headings and breadcrumbs must use raw menu values from the backend navbar data instead of frontend translation keys.

## Scope

- Resolve current page heading and breadcrumb labels from navbar section/item values.
- Keep route fallbacks explicit for pages that are not in the dynamic navbar.
- Update admin shared breadcrumbs and user-facing dashboard pages that currently hard-code translated nav labels.
- Validate with frontend type checking.
