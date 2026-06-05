# 实现后台返佣查询和关系变更存储逻辑

## Scope

- Add storage methods for overview, relation list, relation update audit, commission list, report data, and CSV rows.
- Enforce relation mutation rules in one transaction.

## Acceptance

- Queries return concrete admin DTOs.
- Relation update rejects missing referrer, self-referrer, system referrer, and cycles.
- Relation update writes audit record and does not mutate historical commissions.
