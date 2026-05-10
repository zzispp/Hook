# Admin i18n Backend Resources

## Goal

Move admin UI translations from frontend static JSON runtime loading to backend-managed resources, and add an admin translation management page.

## Scope

- Dynamic backend resource loading for the `admin` i18n namespace.
- Backend tables, storage, service, API, RBAC, baseline seed.
- Frontend resource injection that keeps existing `t()` usage.
- Admin translation management page with value and language tabs.

## Out of Scope

- Dynamic translation for non-admin namespaces.
- Translating DB business configuration values such as menu titles, API names, role names, model names, or billing group names.

