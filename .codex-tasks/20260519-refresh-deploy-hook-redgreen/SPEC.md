# Refresh Deploy Hook Redgreen Spec

Deploy the current Hook code by local frontend build plus local Linux x86_64 cross-compiled backend. Initialize red-green runtime if absent, preserve production secrets, run `migration refresh`, switch Nginx to the refreshed healthy slot, and verify public access plus origin restrictions.
