import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

export function failedSummary(results) {
  return `failed scenarios: ${results.filter((item) => !item.ok).map((item) => item.label).join(', ')}`;
}

export function writeResults(taskDir, results) {
  const rawDir = join(taskDir, 'raw');
  mkdirSync(rawDir, { recursive: true });
  writeFileSync(join(rawDir, 'results.json'), `${JSON.stringify(results, null, 2)}\n`);
}

export function redactEvidence(value, ctx) {
  return JSON.parse(redactText(JSON.stringify(value ?? null), ctx));
}

export function redactText(value, ctx) {
  return String(value)
    .replaceAll(ctx?.cooldownSecrets?.msutoolsKey || 'never-match-msutools', '[redacted-msutools-key]')
    .replaceAll(ctx?.cooldownSecrets?.ekan8Key || 'never-match-ekan8', '[redacted-ekan8-key]')
    .replace(/sk-[A-Za-z0-9_-]+/g, 'sk-[redacted]');
}

