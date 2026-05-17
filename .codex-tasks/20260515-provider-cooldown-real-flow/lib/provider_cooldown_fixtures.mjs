export { fixtureIds, groupCode, uniqueRunId } from './provider_cooldown_constants.mjs';
export { cleanupProviderCooldownFixtures } from './provider_cooldown_cleanup.mjs';
export { prepareProviderCooldownSchema, ensureAdminProviderCooldownApis } from './provider_cooldown_schema.mjs';
export { seedProviderCooldownFixtures } from './provider_cooldown_seed.mjs';
export { applyProviderCooldownSettings, restoreSystemSettings, systemSettingsSnapshot } from './provider_cooldown_settings.mjs';

import { fixtureIds, groupCode } from './provider_cooldown_constants.mjs';

export function testTokenPayload() {
  return {
    name: `Provider cooldown real ${Date.now()}`,
    token_type: 'user',
    user_id: fixtureIds.user,
    group_code: groupCode,
    model_access_mode: 'all',
    allowed_model_ids: [],
    rate_limit_rpm: 0,
  };
}
