import assert from 'node:assert/strict';

import { copyText, createUuid } from '../../apps/hook_frontend/src/utils/browser-compat.ts';
import { fetchModelsDevData } from '../../apps/hook_frontend/src/utils/models-dev.ts';

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

function replaceGlobal(name, value) {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
  Object.defineProperty(globalThis, name, { configurable: true, value });
  return () => {
    if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    else delete globalThis[name];
  };
}

function createLegacyDocument() {
  const state = { command: null, value: null };
  const body = { appendChild(element) { state.value = element.value; } };
  const document = {
    body,
    createElement() {
      return {
        style: {},
        value: '',
        setAttribute() {},
        select() {},
        remove() {},
      };
    },
    execCommand(command) {
      state.command = command;
      return true;
    },
  };
  return { document, state };
}

function testUuidFallback() {
  const restore = replaceGlobal('crypto', {
    getRandomValues(values) {
      values.fill(7);
      return values;
    },
  });
  try {
    assert.match(createUuid(), UUID_PATTERN);
  } finally {
    restore();
  }
}

async function testLegacyClipboard() {
  const { document, state } = createLegacyDocument();
  const restoreNavigator = replaceGlobal('navigator', {});
  const restoreDocument = replaceGlobal('document', document);
  try {
    await copyText('token-value');
    assert.equal(state.command, 'copy');
    assert.equal(state.value, 'token-value');
  } finally {
    restoreDocument();
    restoreNavigator();
  }
}

async function testNativeClipboard() {
  let copied;
  const restoreNavigator = replaceGlobal('navigator', {
    clipboard: { async writeText(value) { copied = value; } },
  });
  try {
    await copyText('https-value');
    assert.equal(copied, 'https-value');
  } finally {
    restoreNavigator();
  }
}

async function testClipboardFailureIsVisible() {
  const restoreNavigator = replaceGlobal('navigator', {});
  const restoreDocument = replaceGlobal('document', undefined);
  try {
    await assert.rejects(copyText('unavailable'), /Clipboard copy failed/);
  } finally {
    restoreDocument();
    restoreNavigator();
  }
}

function jsonResponse(payload, status = 200) {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { 'Content-Type': 'application/json' },
  });
}

async function testModelsDevRequestAndOfficialProviders() {
  let requestInput;
  let requestInit;
  const request = async (input, init) => {
    requestInput = input;
    requestInit = init;
    return jsonResponse({
      openai: { name: 'OpenAI', models: { 'gpt-test': { name: 'GPT Test' } } },
      community: {
        name: 'Community',
        official: true,
        models: { 'community-test': { name: 'Community Test' } },
      },
    });
  };

  const data = await fetchModelsDevData(request);

  assert.equal(requestInput, 'https://models.dev/api.json');
  assert.deepEqual(requestInit, { headers: { Accept: 'application/json' } });
  assert.equal(data.openai.official, true);
  assert.equal(data.community.official, false);
  assert.equal(data.openai.models['gpt-test'].name, 'GPT Test');
}

async function testModelsDevHttpFailureIsVisible() {
  const request = async () => jsonResponse({ message: 'unavailable' }, 503);
  await assert.rejects(fetchModelsDevData(request), /models\.dev request failed: HTTP 503/);
}

async function testModelsDevPayloadValidation() {
  const invalidPayloads = [
    { payload: [], message: /response must be a provider object/ },
    { payload: { openai: null }, message: /provider "openai" must be an object/ },
    {
      payload: { openai: { models: [] } },
      message: /provider "openai" models must be an object/,
    },
    {
      payload: { openai: { models: { broken: null } } },
      message: /model "openai\/broken" must be an object/,
    },
  ];

  for (const { payload, message } of invalidPayloads) {
    const request = async () => jsonResponse(payload);
    await assert.rejects(fetchModelsDevData(request), message);
  }
}

await testUuidFallback();
await testLegacyClipboard();
await testNativeClipboard();
await testClipboardFailureIsVisible();
await testModelsDevRequestAndOfficialProviders();
await testModelsDevHttpFailureIsVisible();
await testModelsDevPayloadValidation();
console.log('REGRESSION_GREEN');
