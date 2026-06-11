# Provider Origin Column

## Goal
Add a provider list column that shows whether a provider was created manually or by quick import.

## Scope
- Persist provider origin on provider records.
- Manual provider creation stores `manual`.
- Quick import provider creation stores `quick_import`.
- Provider list API returns the origin.
- Admin provider table displays the origin with backend-seeded i18n labels.
