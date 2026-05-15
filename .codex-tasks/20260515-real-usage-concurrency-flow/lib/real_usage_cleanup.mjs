import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { fixtureIds, fixtureUserIds, groupCode } from './real_usage_fixtures.mjs';

export function cleanupRealUsageFixtures(db, tokenIds) {
  if (!tableExists(db, 'api_tokens')) {
    return;
  }
  const allTokenIds = [...new Set([...tokenIds, ...tokenIdsForGroup(db)])];
  const requestIds = requestIdsForTokens(db, allTokenIds);
  deleteRequestRows(db, requestIds);
  deleteFixtureRows(db, allTokenIds);
}

function requestIdsForTokens(db, tokenIds) {
  if (tokenIds.length === 0) {
    return [];
  }
  return db
    .rows(`select distinct request_id from request_candidates where token_id in (${tokenIds.map(q).join(',')}) order by request_id;`)
    .map(([requestId]) => requestId);
}

function deleteRequestRows(db, requestIds) {
  if (requestIds.length === 0) {
    return;
  }
  db.exec(`
delete from request_candidates where request_id in (${requestIds.map(q).join(',')});
delete from request_records where request_id in (${requestIds.map(q).join(',')});
delete from wallet_transactions where link_type = 'llm_request_record' and link_id in (${requestIds.map(q).join(',')});`);
}

function deleteFixtureRows(db, tokenIds) {
  db.exec(`delete from usage_flush_batches where id like 'real-usage-%';`);
  if (tokenIds.length > 0) {
    db.exec(`delete from api_tokens where id in (${tokenIds.map(q).join(',')});`);
  }
  db.exec(`
delete from billing_group_models where group_code = ${q(groupCode)};
delete from billing_group_providers where group_code = ${q(groupCode)};
delete from provider_models where provider_id in (${q(fixtureIds.providerHook)}, ${q(fixtureIds.providerEkan8)});
delete from provider_api_keys where provider_id in (${q(fixtureIds.providerHook)}, ${q(fixtureIds.providerEkan8)});
delete from provider_endpoints where provider_id in (${q(fixtureIds.providerHook)}, ${q(fixtureIds.providerEkan8)});
delete from providers where id in (${q(fixtureIds.providerHook)}, ${q(fixtureIds.providerEkan8)});
delete from global_models where id = ${q(fixtureIds.model)};
delete from billing_groups where code = ${q(groupCode)};
delete from wallet_transactions where wallet_id in (select id from wallets where user_id in (${fixtureUserIds.map(q).join(',')}));
delete from wallets where user_id in (${fixtureUserIds.map(q).join(',')});
update users set is_active = false, is_deleted = true, updated_at = now()
where id in (${fixtureUserIds.map(q).join(',')});`);
}

function tokenIdsForGroup(db) {
  return db.rows(`select id from api_tokens where group_code = ${q(groupCode)} order by id;`).map(([id]) => id);
}

function tableExists(db, table) {
  return db.scalar(`select to_regclass(${q(`public.${table}`)});`) === table;
}
