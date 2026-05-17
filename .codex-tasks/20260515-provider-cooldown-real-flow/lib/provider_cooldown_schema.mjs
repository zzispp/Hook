import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

import { providerCooldownPermissionIds } from './provider_cooldown_constants.mjs';

export function prepareProviderCooldownSchema(db) {
  db.exec(`
alter table system_settings
  add column if not exists provider_cooldown_policy text not null default '{"window_seconds":0,"rules":[]}';

create table if not exists provider_cooldowns (
  provider_id varchar(36) primary key references providers(id) on delete cascade,
  provider_name_snapshot varchar(100) not null,
  status_code integer not null,
  observed_count bigint not null,
  threshold_count bigint not null,
  window_seconds bigint not null,
  cooldown_seconds bigint not null,
  triggered_at timestamptz not null,
  cooldown_until timestamptz not null,
  released_at timestamptz null,
  request_id varchar(64) not null,
  candidate_index integer not null,
  retry_index integer not null,
  endpoint_id varchar(36) null,
  endpoint_name_snapshot varchar(50) null,
  key_id varchar(36) null,
  key_name_snapshot varchar(100) null,
  error_type varchar(100) null,
  error_message text null,
  error_code varchar(120) null,
  error_param varchar(160) null,
  created_at timestamptz not null,
  updated_at timestamptz not null
);

create index if not exists index_provider_cooldowns_by_until on provider_cooldowns(cooldown_until);
create index if not exists index_provider_cooldowns_by_status on provider_cooldowns(status_code);
`);
}

export function ensureAdminProviderCooldownApis(db) {
  db.exec(`
insert into api_permissions (id, code, method, path_pattern, name, enabled, system, created_at, updated_at)
values
  (${q(providerCooldownPermissionIds.read)}, 'provider_cooldowns_read', 'GET', '/api/admin/provider-cooldowns', '提供商冷却列表', true, true, now(), now()),
  (${q(providerCooldownPermissionIds.release)}, 'provider_cooldowns_release', 'POST', '/api/admin/provider-cooldowns/{provider_id}/release', '解除提供商冷却', true, true, now(), now())
on conflict (code) do update set
  method = excluded.method,
  path_pattern = excluded.path_pattern,
  name = excluded.name,
  enabled = true,
  system = true,
  updated_at = now();

insert into role_api_permissions (role_code, api_permission_id, created_at, updated_at)
select 'admin', id, now(), now()
from api_permissions
where code in ('provider_cooldowns_read', 'provider_cooldowns_release')
on conflict do nothing;

insert into menu_api_permissions (menu_item_id, api_permission_id, created_at, updated_at)
select mi.id, ap.id, now(), now()
from menu_items mi
cross join api_permissions ap
where mi.code = 'admin_providers'
  and ap.code in ('provider_cooldowns_read', 'provider_cooldowns_release')
on conflict do nothing;`);
}
