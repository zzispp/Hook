import { assertEqual, assertIncludes } from './assertions.mjs';
import { providerNames, modelNames } from './fixtures.mjs';
import { assertConversion, claudeBody, geminiBody, openAiBody, paths, proxyCall, successRow } from './proxy_client.mjs';

export async function runFormatConversion(env) {
  const failures = [];
  await runConversionStep(failures, 'openai to claude', () => runOpenAiToClaude(env));
  await runConversionStep(failures, 'openai to gemini', () => runOpenAiToGemini(env));
  await runConversionStep(failures, 'claude to openai', () => runClaudeToOpenAi(env));
  await runConversionStep(failures, 'gemini exact', () => runGeminiExact(env));
  await runConversionStep(failures, 'gemini stream exact', () => runGeminiStreamExact(env));
  if (failures.length > 0) {
    throw new Error(`format conversion failed sub-scenario(s): ${failures.map((item) => item.label).join(', ')}`);
  }
}

async function runConversionStep(failures, label, action) {
  try {
    await action();
    console.log(`conversion passed: ${label}`);
  } catch (error) {
    failures.push({ label, error });
    console.error(`conversion failed: ${label}: ${error.message}`);
  }
}

async function runOpenAiToClaude(env) {
  const names = modelNames(env.ctx);
  const request = {
    path: paths.openaiChat,
    clientFormat: 'openai_chat',
    model: names.claude,
    matchText: 'hook-claude-convert',
    body: openAiBody(names.claude, 'hook-claude-convert'),
  };
  assertConversion((await proxyCall(env.ctx, env.db, 'openai to claude conversion', request)).trace, providerNames.claude, 'claude_chat');
  const stream = { ...request, matchText: 'hook-claude-stream', body: openAiBody(names.claude, 'hook-claude-stream', true) };
  const result = await proxyCall(env.ctx, env.db, 'openai stream to claude conversion', stream);
  assertConversion(result.trace, providerNames.claude, 'claude_chat');
  env.assertStreamSuccess(result.trace, true);
  assertIncludes(result.text, 'data:', 'stream conversion should return SSE');
}

async function runOpenAiToGemini(env) {
  const names = modelNames(env.ctx);
  const request = {
    path: paths.openaiChat,
    clientFormat: 'openai_chat',
    model: names.gemini,
    matchText: 'hook-gemini-convert',
    body: openAiBody(names.gemini, 'hook-gemini-convert'),
  };
  assertConversion((await proxyCall(env.ctx, env.db, 'openai to gemini conversion', request)).trace, providerNames.gemini, 'gemini_chat');
}

async function runClaudeToOpenAi(env) {
  const names = modelNames(env.ctx);
  const request = {
    path: paths.claudeMessages,
    clientFormat: 'claude_chat',
    model: names.gpt,
    matchText: 'hook-openai-convert',
    body: claudeBody(names.gpt, 'hook-openai-convert'),
  };
  assertConversion((await proxyCall(env.ctx, env.db, 'claude to openai conversion', request)).trace, providerNames.openai, 'openai_chat');
}

async function runGeminiExact(env) {
  const names = modelNames(env.ctx);
  const request = {
    path: paths.gemini(names.gemini),
    clientFormat: 'gemini_chat',
    model: names.gemini,
    matchText: 'hook-gemini-exact',
    body: geminiBody('hook-gemini-exact'),
  };
  const success = successRow((await proxyCall(env.ctx, env.db, 'gemini exact', request)).trace);
  assertEqual(success.provider_name, providerNames.gemini, 'Gemini exact should use Ekan8');
  assertEqual(success.needs_conversion, 'false', 'Gemini exact should not convert');
}

async function runGeminiStreamExact(env) {
  const names = modelNames(env.ctx);
  const request = {
    path: paths.gemini(names.gemini, true),
    clientFormat: 'gemini_chat',
    model: names.gemini,
    matchText: 'hook-gemini-stream-exact',
    body: geminiBody('hook-gemini-stream-exact'),
  };
  const result = await proxyCall(env.ctx, env.db, 'gemini stream exact', request);
  env.assertStreamSuccess(result.trace, false);
}
