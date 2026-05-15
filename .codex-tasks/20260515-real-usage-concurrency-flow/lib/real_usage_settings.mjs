import { assert } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

export function systemSettingsSnapshot(db) {
  const [row] = db.rows(`
select request_record_level, request_record_retention_days::text, request_record_payload_retention_days::text,
  record_request_headers::text, record_request_body::text, record_response_body::text,
  max_request_body_size_kb::text, max_response_body_size_kb::text, scheduling_mode
from system_settings where id = 'global';`);
  assert(row, 'system_settings global row should exist');
  return {
    request_record_level: row[0],
    request_record_retention_days: Number(row[1]),
    request_record_payload_retention_days: Number(row[2]),
    record_request_headers: row[3] === 't',
    record_request_body: row[4] === 't',
    record_response_body: row[5] === 't',
    max_request_body_size_kb: Number(row[6]),
    max_response_body_size_kb: Number(row[7]),
    scheduling_mode: row[8],
  };
}

export function applyRealUsageSettings(db) {
  db.exec(`
update system_settings
set request_record_level = 'full',
    record_request_headers = true,
    record_request_body = true,
    record_response_body = true,
    max_request_body_size_kb = 5120,
    max_response_body_size_kb = 5120,
    scheduling_mode = 'load_balance',
    updated_at = now()
where id = 'global';`);
}

export function restoreSystemSettings(db, snapshot) {
  if (!snapshot) {
    return;
  }
  db.exec(`
update system_settings
set request_record_level = ${q(snapshot.request_record_level)},
    request_record_retention_days = ${snapshot.request_record_retention_days},
    request_record_payload_retention_days = ${snapshot.request_record_payload_retention_days},
    record_request_headers = ${snapshot.record_request_headers ? 'true' : 'false'},
    record_request_body = ${snapshot.record_request_body ? 'true' : 'false'},
    record_response_body = ${snapshot.record_response_body ? 'true' : 'false'},
    max_request_body_size_kb = ${snapshot.max_request_body_size_kb},
    max_response_body_size_kb = ${snapshot.max_response_body_size_kb},
    scheduling_mode = ${q(snapshot.scheduling_mode)},
    updated_at = now()
where id = 'global';`);
}
