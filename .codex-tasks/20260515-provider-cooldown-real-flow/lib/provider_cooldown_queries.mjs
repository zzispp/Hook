import { q } from '../../20260512-real-proxy-cache-flow/lib/db.mjs';

export function activeCooldownRow(db, providerId) {
  const [row] = db.rows(`
select provider_id, provider_name_snapshot, status_code::text, observed_count::text,
  threshold_count::text, window_seconds::text, cooldown_seconds::text,
  request_id, candidate_index::text, retry_index::text, coalesce(error_type, ''),
  coalesce(error_message, ''), coalesce(error_code, ''), coalesce(error_param, '')
from provider_cooldowns
where provider_id = ${q(providerId)}
  and released_at is null
  and cooldown_until > now();`);
  if (!row) return null;
  return {
    provider_id: row[0],
    provider_name: row[1],
    status_code: row[2],
    observed_count: row[3],
    threshold_count: row[4],
    window_seconds: row[5],
    cooldown_seconds: row[6],
    request_id: row[7],
    candidate_index: row[8],
    retry_index: row[9],
    error_type: row[10],
    error_message: row[11],
    error_code: row[12],
    error_param: row[13],
  };
}

