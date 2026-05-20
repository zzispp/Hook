import type { ProviderModelBinding } from 'src/types/provider';

const TEST_MESSAGE = 'Hello! This is a test message.';
const TEST_IMAGE_PROMPT = 'A simple solid red image filling the entire frame, uniform bright red color, minimalistic.';
const JSON_INDENT_SIZE = 2;
const TEST_MAX_TOKENS = 30;
const TEST_TEMPERATURE = 0.7;

export function defaultModelTestHeaders() {
  return JSON.stringify({}, null, JSON_INDENT_SIZE);
}

export function defaultModelTestBody(binding: ProviderModelBinding, apiFormat = 'openai_chat') {
  return JSON.stringify(testBody(modelTestName(binding), apiFormat), null, JSON_INDENT_SIZE);
}

export function parseJsonObjectDraft(
  draft: string,
  emptyValue: Record<string, unknown> | null
): { value: Record<string, unknown> | null; error: string | null } {
  const normalized = draft.trim();
  if (!normalized) {
    return { value: emptyValue, error: null };
  }
  try {
    return parsedObject(JSON.parse(normalized));
  } catch (error) {
    return { value: null, error: error instanceof Error ? error.message : 'Invalid JSON' };
  }
}

export function formatJsonDraft(draft: string) {
  return JSON.stringify(JSON.parse(draft), null, JSON_INDENT_SIZE);
}

function parsedObject(parsed: unknown) {
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
    return { value: null, error: 'JSON value must be an object' };
  }
  return { value: parsed as Record<string, unknown>, error: null };
}

function modelTestName(binding: ProviderModelBinding) {
  return binding.provider_model_mapping?.name ?? binding.provider_model_name;
}

function testBody(model: string, apiFormat: string) {
  if (apiFormat.startsWith('gemini_')) {
    return geminiBody(model);
  }
  if (apiFormat.startsWith('claude_')) {
    return claudeBody(model);
  }
  if (apiFormat === 'openai_cli' || apiFormat === 'openai_compact') {
    return openAiResponsesBody(model, apiFormat !== 'openai_compact');
  }
  if (apiFormat === 'openai_image') {
    return openAiImageBody(model);
  }
  return openAiChatBody(model);
}

function openAiChatBody(model: string) {
  return {
    model,
    messages: [{ role: 'user', content: TEST_MESSAGE }],
    max_tokens: TEST_MAX_TOKENS,
    temperature: TEST_TEMPERATURE,
    stream: true,
  };
}

function openAiResponsesBody(model: string, stream: boolean) {
  return {
    model,
    input: [{ role: 'user', content: TEST_MESSAGE }],
    max_output_tokens: TEST_MAX_TOKENS,
    temperature: TEST_TEMPERATURE,
    stream,
  };
}

function openAiImageBody(model: string) {
  return {
    model,
    prompt: TEST_IMAGE_PROMPT,
  };
}

function claudeBody(model: string) {
  return {
    model,
    messages: [{ role: 'user', content: TEST_MESSAGE }],
    max_tokens: TEST_MAX_TOKENS,
    temperature: TEST_TEMPERATURE,
    stream: true,
  };
}

function geminiBody(model: string) {
  return {
    model,
    contents: [{ role: 'user', parts: [{ text: TEST_MESSAGE }] }],
    generationConfig: {
      maxOutputTokens: TEST_MAX_TOKENS,
      temperature: TEST_TEMPERATURE,
    },
    stream: true,
  };
}
