# Affiliate Operator FK Fix

## Goal
Allow admin affiliate relation changes from virtual system admin users without violating affiliate_relation_changes.operator_user_id foreign key, while preserving operator audit for database-backed users.
