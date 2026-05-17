import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

import { fixtureIds, groupCode, providerCooldownPermissionIds } from './provider_cooldown_constants.mjs';

export function cleanupProviderCooldownFixtures(db, tokenIds = []) {
  const tokenIdList = tokenIds.map(q).join(',') || 'null';
  db.exec(`
delete from api_tokens where id in (${tokenIdList});
delete from request_candidates where request_id in (select id from request_records where token_id in (${tokenIdList}));
delete from request_records where token_id in (${tokenIdList});
delete from provider_cooldowns where provider_id in (${q(fixtureIds.providerMsutools)}, ${q(fixtureIds.providerEkan8)});
delete from billing_group_models where id = ${q(fixtureIds.groupModel)};
delete from billing_group_providers where id in (${q(fixtureIds.groupBindingMsutools)}, ${q(fixtureIds.groupBindingEkan8)});
delete from provider_models where id in (${q(fixtureIds.modelBindingMsutools)}, ${q(fixtureIds.modelBindingEkan8)});
delete from provider_api_keys where id in (${q(fixtureIds.keyMsutools)}, ${q(fixtureIds.keyEkan8)});
delete from provider_endpoints where id in (${q(fixtureIds.endpointMsutools)}, ${q(fixtureIds.endpointEkan8)});
delete from providers where id in (${q(fixtureIds.providerMsutools)}, ${q(fixtureIds.providerEkan8)});
delete from billing_groups where id = ${q(fixtureIds.group)} or code = ${q(groupCode)};
delete from wallets where id = ${q(fixtureIds.wallet)} or user_id = ${q(fixtureIds.user)};
delete from users where id = ${q(fixtureIds.user)};
delete from global_models where id = ${q(fixtureIds.model)};
delete from api_permissions
where id in (${q(providerCooldownPermissionIds.read)}, ${q(providerCooldownPermissionIds.release)})
  and code in ('provider_cooldowns_read', 'provider_cooldowns_release');
delete from menu_api_permissions where api_permission_id not in (select id from api_permissions);
delete from role_api_permissions where api_permission_id not in (select id from api_permissions);`);
}
