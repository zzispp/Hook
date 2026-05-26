# Progress

## 2026-05-26

- Started implementation from approved plan.

- Optimized request record cost details for admin scanning: summary metrics now lead with upstream cost, billed amount, and profit; detailed prices are shown in a compact upstream-vs-global comparison grid.
- Reused ProviderModelCostDialogFields from the model cost dialog to keep the dialog file under the 300-line project limit and keep the global-price fill button fix centralized.
- Validation: pnpm lint:frontend passed; pnpm build:frontend passed.
- Extended the admin dashboard business metrics: backend aggregates upstream cost snapshots, profit, and profit rate; frontend shows admin-only upstream cost and profit-rate KPI cards, adds cost/profit series to the trend chart, and appends upstream cost/profit rate to breakdown rows such as provider distribution.
- Activity grid tooltips now use true upstream cost for admins and no longer label `base_cost` as operating cost; non-admin dashboard responses hide upstream cost values.
- Validation: cargo check -q -p storage passed; cargo check -q -p dashboard passed; pnpm lint:frontend passed.
