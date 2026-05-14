import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';
import { ids } from './billing_access_ids.mjs';

export function systemSettingsSnapshot(db) {
  const [row] = db.rows(`
select request_record_level, record_request_headers::text, record_request_body::text,
  record_response_body::text, max_request_body_size_kb::text, max_response_body_size_kb::text,
  scheduling_mode
from system_settings where id = 'global';`);
  if (!row) throw new Error('system_settings global row should exist');
  return Object.freeze({
    request_record_level: row[0],
    record_request_headers: row[1] === 't',
    record_request_body: row[2] === 't',
    record_response_body: row[3] === 't',
    max_request_body_size_kb: Number(row[4]),
    max_response_body_size_kb: Number(row[5]),
    scheduling_mode: row[6],
  });
}

export function applyRecordingSettings(db) {
  db.exec(`
update system_settings
set request_record_level = 'full',
    record_request_headers = true,
    record_request_body = true,
    record_response_body = true,
    max_request_body_size_kb = 5120,
    max_response_body_size_kb = 5120,
    updated_at = now()
where id = 'global';`);
}

export function ensureRequestSnapshotColumns(db) {
  db.exec(`
alter table request_records add column if not exists user_id_snapshot varchar(36);
alter table request_records add column if not exists username_snapshot varchar(100);
alter table request_records add column if not exists token_name_snapshot varchar(100);
alter table request_records add column if not exists token_prefix_snapshot varchar(32);
alter table request_records add column if not exists model_name_snapshot varchar(100);
alter table request_records add column if not exists provider_name_snapshot varchar(100);
alter table request_records add column if not exists provider_key_name_snapshot varchar(100);
alter table request_records add column if not exists provider_key_preview_snapshot varchar(32);
alter table request_candidates add column if not exists provider_name_snapshot varchar(100);
alter table request_candidates add column if not exists endpoint_name_snapshot varchar(50);
alter table request_candidates add column if not exists key_name_snapshot varchar(100);
alter table request_candidates add column if not exists key_preview_snapshot varchar(32);`);
}

export function restoreSystemSettings(db, snapshot) {
  db.exec(`
update system_settings
set request_record_level = ${q(snapshot.request_record_level)},
    record_request_headers = ${snapshot.record_request_headers ? 'true' : 'false'},
    record_request_body = ${snapshot.record_request_body ? 'true' : 'false'},
    record_response_body = ${snapshot.record_response_body ? 'true' : 'false'},
    max_request_body_size_kb = ${snapshot.max_request_body_size_kb},
    max_response_body_size_kb = ${snapshot.max_response_body_size_kb},
    scheduling_mode = ${q(snapshot.scheduling_mode)},
    updated_at = now()
where id = 'global';`);
}

export function setSchedulingMode(db, mode) {
  db.exec(`update system_settings set scheduling_mode = ${q(mode)}, updated_at = now() where id = 'global';`);
}

export function setAccessProviders(db, patch) {
  const active = activeProviderIds(patch);
  db.exec(`
update providers set is_active = false, updated_at = now() where id in (${providerIds().map(q).join(',')});
update providers set is_active = true, updated_at = now() where id in (${active.map(q).join(',')});
update providers set priority = ${Number(patch.primaryPriority ?? 10)}, updated_at = now()
where id in (${[ids.providerPrimaryA, ids.providerPrimaryB].map(q).join(',')});
update providers set priority = ${Number(patch.brokenPriority ?? 0)}, max_retries = ${Number(patch.brokenRetries ?? 1)}, updated_at = now()
where id = ${q(ids.providerBroken)};
update provider_endpoints set max_retries = ${Number(patch.brokenRetries ?? 1)}, updated_at = now()
where id = ${q(ids.endpointBroken)};
update providers set priority = ${Number(patch.slowPriority ?? 0)}, request_timeout_seconds = ${Number(patch.slowTimeout ?? 0.2)}, updated_at = now()
where id = ${q(ids.providerSlow)};`);
}

export function setPrimaryKeyPriorities(db, primaryA, primaryB) {
  db.exec(`
update provider_api_keys set internal_priority = ${Number(primaryA)}, updated_at = now() where id = ${q(ids.keyPrimaryA)};
update provider_api_keys set internal_priority = ${Number(primaryB)}, updated_at = now() where id = ${q(ids.keyPrimaryB)};`);
}

function activeProviderIds(patch) {
  const selected = [];
  if (patch.primaryA !== false) selected.push(ids.providerPrimaryA);
  if (patch.primaryB) selected.push(ids.providerPrimaryB);
  if (patch.ekan8) selected.push(ids.providerEkan8);
  if (patch.broken) selected.push(ids.providerBroken);
  if (patch.slow) selected.push(ids.providerSlow);
  return selected.length ? selected : [ids.providerPrimaryA];
}

function providerIds() {
  return [ids.providerPrimaryA, ids.providerPrimaryB, ids.providerEkan8, ids.providerBroken, ids.providerSlow];
}
