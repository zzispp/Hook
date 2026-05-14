import { groupCodes, ids } from './billing_access_ids.mjs';
import { setAccessProviders, setSchedulingMode } from './billing_access_db_control.mjs';
import {
  openAiChatRequest,
  proxyExchange,
  successCandidate,
  tokenSnapshot,
  walletSnapshot,
  walletTransactions,
} from './billing_access_client.mjs';
import {
  expect,
  expectDecimalClose,
  expectEqual,
  expectGreaterThan,
  expectIncludes,
  expectNoTrace,
  expectStatus,
  expectSuccess,
} from './billing_access_assertions.mjs';

export async function disabledTokenRejected(state) {
  await prepareSingleProvider(state);
  const result = await exchange(state, ids.tokenDisabled, state.tokenValues.disabledToken, 'disabled-token');
  expectStatus(result, 401, `disabled token should be HTTP 401 ${brief(result)}`);
  expectNoTrace(result, 'disabled token should fail before request_candidates');
  return result;
}

export async function disabledUserRejected(state) {
  await prepareSingleProvider(state);
  const result = await exchange(state, ids.tokenDisabledUser, state.tokenValues.disabledUser, 'disabled-user');
  expectStatus(result, 403, `disabled user should match New API HTTP 403 ${brief(result)}`);
  expectNoTrace(result, 'disabled user should fail before upstream scheduling');
  return result;
}

export async function tokenQuotaRejected(state) {
  await prepareSingleProvider(state);
  const result = await exchange(state, ids.tokenQuota, state.tokenValues.tokenQuota, 'token-quota');
  expectStatus(result, 403, `exhausted token quota should match New API HTTP 403 ${brief(result)}`);
  expectIncludes(result.body, 'pre_consume_token_quota_failed', 'token quota body should match New API code');
  expectNoTrace(result, 'token quota should fail before request_candidates');
  return result;
}

export async function walletQuotaRejected(state) {
  await prepareSingleProvider(state);
  const result = await exchange(state, ids.tokenWallet, state.tokenValues.walletQuota, 'wallet-quota');
  expectStatus(result, 403, `empty wallet should match New API HTTP 403 ${brief(result)}`);
  expectIncludes(result.body, 'insufficient_user_quota', 'wallet quota body should match New API code');
  expectNoTrace(result, 'wallet quota should fail before request_candidates');
  return result;
}

export async function billingMultiplierAndTokenUsage(state) {
  await prepareSingleProvider(state);
  const tokenBefore = tokenSnapshot(state.db, ids.tokenActive);
  const walletBefore = walletSnapshot(state.db, ids.walletActive);
  const result = await exchange(state, ids.tokenActive, state.tokenValues.active, 'billing');
  expectSuccess(result, `billing request should succeed ${brief(result)}`);
  const success = successCandidate(result.trace);
  expect(success, 'billing request should record one success candidate');
  expectDecimalClose(success.billing_multiplier, 2.5, 'candidate billing multiplier should be the token group multiplier');
  expectGreaterThan(success.base_cost, 0, 'candidate base cost should be positive');
  expectDecimalClose(success.total_cost, Number(success.base_cost) * 2.5, 'candidate total cost should be base * multiplier');
  const tokenAfter = tokenSnapshot(state.db, ids.tokenActive);
  expectDecimalClose(Number(tokenAfter.used_quota) - Number(tokenBefore.used_quota), success.total_cost, 'api token used_quota should increase by total_cost');
  state.artifacts.billing = { result, tokenBefore, tokenAfter, walletBefore, totalCost: success.total_cost };
  return state.artifacts.billing;
}

export async function walletLedgerCharged(state) {
  const billing = state.artifacts.billing;
  expect(billing, 'billing scenario must run before wallet ledger check');
  const walletAfter = walletSnapshot(state.db, ids.walletActive);
  const transactions = walletTransactions(state.db, ids.walletActive);
  expect(Number(walletAfter.recharge_balance) < Number(billing.walletBefore.recharge_balance), 'wallet recharge balance should decrease after billed request');
  expect(Number(walletAfter.total_consumed) > Number(billing.walletBefore.total_consumed), 'wallet total_consumed should increase after billed request');
  expectGreaterThan(transactions.length, 0, 'wallet consume ledger should be created');
  const consume = transactions.find((item) => item.category === 'consume' && item.reasonCode === 'llm_model_usage');
  expect(consume, 'wallet ledger should include LLM consume category and reason');
  expectEqual(consume.linkType, 'llm_request_record', 'wallet ledger link_type should point to request record');
  expectEqual(consume.linkId, billing.result.requestId, 'wallet ledger link_id should equal request_id');
  expectDecimalClose(Math.abs(Number(consume.amount)), billing.totalCost, 'wallet consume amount should equal billed total cost');
  assertLedgerSnapshot(consume.snapshot, billing);
  return { walletBefore: billing.walletBefore, walletAfter, transactions };
}

function assertLedgerSnapshot(snapshot, billing) {
  const success = successCandidate(billing.result.trace);
  expect(snapshot, 'wallet ledger description should contain JSON snapshot');
  expectEqual(snapshot.kind, 'llm_model_usage', 'wallet ledger snapshot kind');
  expectEqual(snapshot.request.request_id, billing.result.requestId, 'wallet ledger snapshot request id');
  expectEqual(snapshot.token.id, ids.tokenActive, 'wallet ledger snapshot token id');
  expectEqual(snapshot.group.code, groupCodes.high, 'wallet ledger snapshot group code');
  expectEqual(snapshot.model.global_model_id, ids.modelOpenai, 'wallet ledger snapshot model id');
  expectEqual(snapshot.provider.id, ids.providerPrimaryA, 'wallet ledger snapshot provider id');
  expectEqual(snapshot.key.id, success.key_id, 'wallet ledger snapshot provider key id');
  expectDecimalClose(snapshot.amounts.total_cost, billing.totalCost, 'wallet ledger snapshot total cost');
  expectDecimalClose(snapshot.amounts.deducted_amount, billing.totalCost, 'wallet ledger snapshot deducted amount');
}

async function prepareSingleProvider(state) {
  setSchedulingMode(state.db, 'fixed_order');
  setAccessProviders(state.db, { primaryA: true });
  await state.clearCaches();
}

async function exchange(state, tokenId, token, marker) {
  return proxyExchange(
    state.ctx,
    state.db,
    tokenId,
    token,
    marker,
    openAiChatRequest(state.models.openai, state.marker(marker)),
  );
}

function brief(result) {
  return JSON.stringify({
    status: result.status,
    requestId: result.requestId,
    trace: result.trace.map((row) => ({
      provider: row.provider_name,
      status: row.status,
      code: row.status_code,
      error: row.error_type || row.error_code,
    })),
    body: result.body,
  });
}
