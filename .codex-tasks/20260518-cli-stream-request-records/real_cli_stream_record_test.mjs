import { spawnSync } from 'node:child_process';
const backendUrl = process.env.HOOK_BACKEND_URL || 'http://127.0.0.1:5566';
const token = requiredEnv('HOOK_TEST_TOKEN');
const model = process.env.HOOK_TEST_MODEL || 'gpt-5.5';
const marker = `cli-stream-record-${Date.now()}`;
const beforeIso = new Date(Date.now() - 1000).toISOString();

const response = await fetch(`${backendUrl}/v1/responses`, {
  method: 'POST',
  headers: {
    authorization: `Bearer ${token}`,
    'content-type': 'application/json',
  },
  body: JSON.stringify({
    model,
    input: [{ role: 'user', content: [{ type: 'input_text', text: `Reply with OK for ${marker}` }] }],
    stream: true,
    max_output_tokens: 32,
  }),
});

if (!response.ok) {
  throw new Error(`stream request failed ${response.status}: ${await response.text()}`);
}

const text = await response.text();
if (!text.includes('response.completed')) {
  throw new Error(`stream response did not include response.completed: ${text.slice(0, 500)}`);
}

const requestId = await waitForRequestId(beforeIso, marker);
const record = await waitForTerminalRecord(requestId);
const candidate = dbRow(`
select status, coalesce(status_code::text,''), coalesce(prompt_tokens::text,''), coalesce(completion_tokens::text,''), coalesce(total_tokens::text,''), coalesce(total_cost::text,''), coalesce(finished_at::text,'')
from request_candidates
where request_id = ${q(requestId)}
order by candidate_index, retry_index
limit 1;`);

console.log(JSON.stringify({ requestId, record, candidate, responseBytes: text.length }, null, 2));

async function waitForRequestId(createdAfter, bodyMarker) {
  const started = Date.now();
  while (Date.now() - started < 15000) {
    const requestId = dbScalar(`
select request_id
from request_records
where client_api_format = 'openai_cli'
  and is_stream = true
  and created_at >= ${q(createdAfter)}
  and request_body like ${q(`%${bodyMarker}%`)}
order by created_at desc
limit 1;`);
    if (requestId) {
      return requestId;
    }
    await sleep(250);
  }
  throw new Error('stream request record was not created');
}

async function waitForTerminalRecord(requestId) {
  const started = Date.now();
  while (Date.now() - started < 15000) {
    const row = dbRow(`
select status, billing_status, coalesce(client_status_code::text,''), coalesce(prompt_tokens::text,''), coalesce(completion_tokens::text,''), coalesce(total_tokens::text,''), coalesce(total_cost::text,''), coalesce(finished_at::text,'')
from request_records
where request_id = ${q(requestId)}
limit 1;`);
    if (!row) {
      throw new Error(`missing request record ${requestId}`);
    }
    if (!['pending', 'streaming'].includes(row.status)) {
      return row;
    }
    await sleep(250);
  }
  throw new Error(`request record stayed non-terminal: ${requestId}`);
}

function dbRow(sql) {
  const [row] = dbRows(sql);
  if (!row) {
    return null;
  }
  return {
    status: row[0],
    billingStatusOrStatusCode: row[1],
    statusCodeOrPromptTokens: row[2],
    promptOrCompletionTokens: row[3],
    completionOrTotalTokens: row[4],
    totalTokensOrCost: row[5],
    totalCostOrFinishedAt: row[6],
    finishedAt: row[7],
  };
}

function dbScalar(sql) {
  return dbRows(sql)[0]?.[0] || '';
}

function dbRows(sql) {
  const result = spawnSync(
    'docker',
    ['exec', '-i', 'hook-postgres', 'psql', '-X', '-q', '-v', 'ON_ERROR_STOP=1', '-At', '-F', '\t', '-U', 'postgres', '-d', 'postgres'],
    { input: sql, encoding: 'utf8', maxBuffer: 1024 * 1024 * 10 },
  );
  if (result.status !== 0) {
    throw new Error(result.stderr || result.stdout);
  }
  if (!result.stdout.trim()) {
    return [];
  }
  return result.stdout.trimEnd().split('\n').map((line) => line.split('\t'));
}

function requiredEnv(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`missing required env ${name}`);
  }
  return value;
}

function q(value) {
  return `'${String(value).replaceAll("'", "''")}'`;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
