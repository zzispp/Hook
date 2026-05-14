import { randomBytes } from 'node:crypto';

export const ids = Object.freeze({
  modelOpenai: '00000000-0000-7000-9300-000000000701',
  modelEkan8: '00000000-0000-7000-9300-000000000702',
  groupHigh: '00000000-0000-7000-9300-000000000801',
  groupLow: '00000000-0000-7000-9300-000000000802',
  providerPrimaryA: '00000000-0000-7000-9300-000000000101',
  providerPrimaryB: '00000000-0000-7000-9300-000000000102',
  providerEkan8: '00000000-0000-7000-9300-000000000103',
  providerBroken: '00000000-0000-7000-9300-000000000104',
  providerSlow: '00000000-0000-7000-9300-000000000105',
  keyPrimaryA: '00000000-0000-7000-9300-000000000201',
  keyPrimaryB: '00000000-0000-7000-9300-000000000202',
  keyEkan8: '00000000-0000-7000-9300-000000000203',
  keyBroken: '00000000-0000-7000-9300-000000000204',
  keySlow: '00000000-0000-7000-9300-000000000205',
  endpointPrimaryA: '00000000-0000-7000-9300-000000000301',
  endpointPrimaryB: '00000000-0000-7000-9300-000000000302',
  endpointEkan8: '00000000-0000-7000-9300-000000000303',
  endpointBroken: '00000000-0000-7000-9300-000000000304',
  endpointSlow: '00000000-0000-7000-9300-000000000305',
  providerModelPrimaryA: '00000000-0000-7000-9300-000000000401',
  providerModelPrimaryB: '00000000-0000-7000-9300-000000000402',
  providerModelEkan8: '00000000-0000-7000-9300-000000000403',
  providerModelBroken: '00000000-0000-7000-9300-000000000404',
  providerModelSlow: '00000000-0000-7000-9300-000000000405',
  userActive: '00000000-0000-7000-9300-000000000501',
  userDisabled: '00000000-0000-7000-9300-000000000502',
  userTokenQuota: '00000000-0000-7000-9300-000000000503',
  userWalletQuota: '00000000-0000-7000-9300-000000000504',
  userRouting: '00000000-0000-7000-9300-000000000505',
  tokenActive: '00000000-0000-7000-9300-000000000601',
  tokenDisabled: '00000000-0000-7000-9300-000000000602',
  tokenDisabledUser: '00000000-0000-7000-9300-000000000603',
  tokenQuota: '00000000-0000-7000-9300-000000000604',
  tokenWallet: '00000000-0000-7000-9300-000000000605',
  tokenRouting: '00000000-0000-7000-9300-000000000606',
  walletActive: '00000000-0000-7000-9300-000000000901',
  walletDisabled: '00000000-0000-7000-9300-000000000902',
  walletTokenQuota: '00000000-0000-7000-9300-000000000903',
  walletQuota: '00000000-0000-7000-9300-000000000904',
  walletRouting: '00000000-0000-7000-9300-000000000905',
});

export const groupCodes = Object.freeze({
  high: 'billing_access_real',
  low: 'billing_access_real_low',
});

export const providerNames = Object.freeze({
  primaryA: 'Billing Access Hook A',
  primaryB: 'Billing Access Hook B',
  ekan8: 'Billing Access Ekan8',
  broken: 'Billing Access Broken',
  slow: 'Billing Access Slow',
});

export function makeTokenValues() {
  return Object.freeze({
    active: randomToken('active'),
    disabledToken: randomToken('disabled-token'),
    disabledUser: randomToken('disabled-user'),
    tokenQuota: randomToken('token-quota'),
    walletQuota: randomToken('wallet-quota'),
    routing: randomToken('routing'),
  });
}

function randomToken(label) {
  return `sk-billing-access-${label}-${randomBytes(18).toString('hex')}`;
}
