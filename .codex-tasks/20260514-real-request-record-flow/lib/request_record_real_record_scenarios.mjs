import { assert, assertEqual, assertIncludes } from '../../20260512-real-proxy-cache-flow/lib/assertions.mjs';
import { invalidRoleOpenAiChatRequest, cancelProxyStream, openAiChatRequest, proxyCall } from './request_record_real_client.mjs';
import {
  getRequestRecord,
  listRequestRecords,
  markForCompression,
  prepareStalePendingRecord,
  prepareStaleStreamingRecord,
  rawCandidatePayloads,
  rawSummaryPayloads,
  waitForRecordStatus,
} from './request_record_real_support.mjs';

export async function structuredUpstreamError(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues, artifacts } = state;
  const result = await proxyCall(ctx, db, tokenValues.openaiOnly, 'invalid role', invalidRoleOpenAiChatRequest(ctx, ctx.models.openai, state.marker('invalid-role')), { expectOk: false });
  const record = await waitForRecordStatus(db, result.requestId, 'failed');
  const failed = result.trace.find((row) => row.status === 'failed');
  assertEqual(result.status, 400, 'invalid upstream request should return 400');
  assert(failed, 'failed candidate should exist');
  assertEqual(failed.error_code, 'invalid_value', 'candidate should extract upstream error code');
  assertEqual(failed.error_param, 'input[0]', 'candidate should extract upstream error param');
  assertIncludes(failed.error_message, 'Invalid value', 'candidate error message should be concrete');
  assertEqual(record.client_status_code, '400', 'main record should keep client status code');
  assertEqual(record.client_error_type, 'upstream_status', 'main record should classify upstream status');
  artifacts.failedForVisibility = result.requestId;
  return { requestId: result.requestId, errorCode: failed.error_code, errorParam: failed.error_param };
}

export async function cancelledStream(state, modelIds) {
  await state.directSchedulingChange(modelIds, 'fixed_order');
  const { ctx, db, tokenValues, artifacts } = state;
  const request = openAiChatRequest(ctx, ctx.models.openai, `${state.marker('cancel-stream')} Write 160 short numbered words.`, true);
  request.body.max_tokens = 180;
  const result = await cancelProxyStream(ctx, db, tokenValues.openaiOnly, 'cancel stream', request);
  const record = await waitForRecordStatus(db, result.requestId, 'cancelled');
  assertEqual(record.client_status_code, '499', 'cancelled stream should keep 499');
  assertEqual(record.termination_origin, 'client', 'cancelled stream should mark client origin');
  assertEqual(record.termination_reason, 'disconnected', 'cancelled stream should mark disconnected reason');
  assertEqual(record.stream_end_reason, 'client_disconnected', 'cancelled stream should mark stream end reason');
  artifacts.cancelledForVisibility = result.requestId;
  return { requestId: result.requestId, status: record.status };
}

export async function compressionRetention(state) {
  markForCompression(state.db, state.artifacts.successForCompression);
  await state.restartBackend();
  await waitForCompression(state);
  const detail = await getRequestRecord(state.ctx, state.adminToken(), state.artifacts.successForCompression);
  assert(detail.request_body?.model, 'compressed request detail should still decode request body');
  assert(detail.client_response_body, 'compressed request detail should still decode client response body');
  return { requestId: state.artifacts.successForCompression };
}

export async function staleSweep(state) {
  const pending = await proxyCall(state.ctx, state.db, state.tokenValues.openaiOnly, 'stale pending seed', openAiChatRequest(state.ctx, state.ctx.models.openai, state.marker('stale-pending')));
  const streaming = await proxyCall(state.ctx, state.db, state.tokenValues.openaiOnly, 'stale streaming seed', openAiChatRequest(state.ctx, state.ctx.models.openai, state.marker('stale-streaming'), true));
  prepareStalePendingRecord(state.db, pending.requestId);
  prepareStaleStreamingRecord(state.db, streaming.requestId);
  await state.restartBackend();
  const pendingRow = await waitForRecordStatus(state.db, pending.requestId, 'failed');
  const streamingRow = await waitForRecordStatus(state.db, streaming.requestId, 'failed');
  assertEqual(pendingRow.client_status_code, '504', 'stale pending should become 504');
  assertEqual(pendingRow.termination_reason, 'pending_timeout', 'stale pending should mark pending timeout');
  assertEqual(streamingRow.client_status_code, '504', 'stale streaming should become 504');
  assertEqual(streamingRow.stream_end_reason, 'stale_streaming_timeout', 'stale streaming should mark streaming timeout');
  return { pending: pending.requestId, streaming: streaming.requestId };
}

export async function requestRecordVisibility(state, modelIds) {
  const list = await listRequestRecords(state.ctx, state.adminToken(), { search: 'User access openaiOnly', status: 'success', model_id: modelIds.openai, limit: 100 });
  assert(list.total >= 100, 'request record list should include concurrent openaiOnly traffic');
  const detail = await getRequestRecord(state.ctx, state.adminToken(), state.artifacts.cancelledForVisibility);
  assertEqual(detail.record.status, 'cancelled', 'detail should expose cancelled status');
  assert(detail.candidates.some((item) => item.status === 'cancelled'), 'detail should expose cancelled candidate');
  const failedDetail = await getRequestRecord(state.ctx, state.adminToken(), state.artifacts.failedForVisibility);
  assert(failedDetail.candidates.some((item) => item.error_code === 'invalid_value'), 'detail should expose error_code');
  return { total: list.total, cancelled: state.artifacts.cancelledForVisibility, failed: state.artifacts.failedForVisibility };
}

async function waitForCompression(state) {
  const started = Date.now();
  while (Date.now() - started < 10_000) {
    const summary = rawSummaryPayloads(state.db, state.artifacts.successForCompression);
    const candidates = rawCandidatePayloads(state.db, state.artifacts.successForCompression);
    const summaryCompressed = Object.values(summary).every((value) => value.includes('__hook_compressed_payload__'));
    const candidateCompressed = candidates.some((row) => row.status === 'success' && row.provider_request_body.includes('__hook_compressed_payload__'));
    if (summaryCompressed && candidateCompressed) return;
    await sleep(250);
  }
  throw new Error(`payload compression did not complete for ${state.artifacts.successForCompression}`);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
