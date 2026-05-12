import { loadContext } from './lib/env.mjs';

const ctx = loadContext();

async function main() {
  await probeOpenAI();
  await probeClaude();
  await probeGemini();
}

async function probeOpenAI() {
  await probeJson('openai', `${ctx.upstreams.openaiBaseUrl}/v1/models`, {
    Authorization: `Bearer ${ctx.secrets.hookPoolKey}`,
  });
}

async function probeClaude() {
  await probeJson('claude', `${ctx.upstreams.claudeBaseUrl}/v1/models`, {
    'x-api-key': ctx.secrets.claudeKey,
    'anthropic-version': '2023-06-01',
  });
}

async function probeGemini() {
  await probeJson('gemini', `${ctx.upstreams.geminiBaseUrl}/v1beta/models?key=${encodeURIComponent(ctx.secrets.geminiKey)}`, {});
}

async function probeJson(label, url, headers) {
  const response = await fetch(url, { headers });
  const text = await response.text();
  console.log(`${label}: status=${response.status}`);
  console.log(text.slice(0, 3000));
}

main().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});

