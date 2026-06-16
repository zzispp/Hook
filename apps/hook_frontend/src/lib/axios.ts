import type { AxiosRequestConfig } from 'axios';

import axios from 'axios';

import { CONFIG } from 'src/global-config';

import { JWT_REFRESH_ENDPOINT } from 'src/auth/context/jwt/constant';
import {
  installJwtInterceptors,
  retryAfterUnauthorized,
} from 'src/auth/context/jwt/axios-interceptors';

// ----------------------------------------------------------------------

export const HTTP_UNAUTHORIZED = 401;

type ApiRequestErrorOptions = {
  status?: number;
  data?: unknown;
};

export class ApiRequestError extends Error {
  readonly status?: number;

  readonly data?: unknown;

  constructor(message: string, options: ApiRequestErrorOptions = {}) {
    super(message);
    this.name = 'ApiRequestError';
    this.status = options.status;
    this.data = options.data;
  }
}

export function isApiRequestError(error: unknown): error is ApiRequestError {
  return error instanceof ApiRequestError;
}

export function isRequestCancelled(error: unknown) {
  return axios.isCancel(error);
}

const axiosInstance = axios.create({
  baseURL: CONFIG.serverUrl,
  headers: {
    'Content-Type': 'application/json',
  },
});

installJwtInterceptors(axiosInstance);

axiosInstance.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (isRequestCancelled(error)) {
      return Promise.reject(error);
    }

    const retried = await retryAfterUnauthorized(error);

    if (retried) {
      return retried;
    }

    const message = requestErrorMessage(error);
    console.error('Axios error:', message);
    return Promise.reject(toApiRequestError(error, message));
  }
);

export default axiosInstance;

// ----------------------------------------------------------------------

export const fetcher = async <T = unknown>(
  args: string | [string, AxiosRequestConfig]
): Promise<T> => {
  try {
    const [url, config] = Array.isArray(args) ? args : [args, {}];

    const res = await axiosInstance.get<T>(url, config);

    return res.data;
  } catch (error) {
    if (isRequestCancelled(error)) {
      throw error;
    }

    console.error('Fetcher failed:', error);
    throw error;
  }
};

function requestErrorMessage(error: unknown) {
  if (axios.isAxiosError(error)) {
    return error.response?.data?.message || error.message || 'Something went wrong!';
  }

  if (error instanceof Error) {
    return error.message || error.name || 'Something went wrong!';
  }

  return 'Something went wrong!';
}

function toApiRequestError(error: unknown, message: string) {
  if (axios.isAxiosError(error)) {
    return new ApiRequestError(message, {
      status: error.response?.status,
      data: error.response?.data,
    });
  }

  return new ApiRequestError(message);
}

// ----------------------------------------------------------------------

export const endpoints = {
  chat: '/api/chat',
  kanban: '/api/kanban',
  calendar: '/api/calendar',
  auth: {
    config: '/api/auth/config',
    me: '/api/auth/me',
    refresh: JWT_REFRESH_ENDPOINT,
    signIn: '/api/auth/sign-in',
    signUp: '/api/auth/sign-up',
    registrationEmailCode: '/api/auth/registration-email-code',
    passwordResetRequest: '/api/auth/password-reset/request',
    passwordResetConfirm: '/api/auth/password-reset/confirm',
    oauthStart: (provider: string) => `/api/auth/oauth/${provider}/start`,
    oauthCallback: (provider: string) => `/api/auth/oauth/${provider}/callback`,
    oauthBindExisting: (provider: string) => `/api/auth/oauth/${provider}/bind-existing`,
    walletNonce: '/api/auth/wallet/nonce',
    walletSignIn: '/api/auth/wallet/sign-in',
    walletRegister: '/api/auth/wallet/register',
  },
  account: {
    profile: '/api/account/profile',
    affiliateSummary: '/api/account/affiliate-summary',
    affiliateReferrals: '/api/account/affiliate/referrals',
    affiliateCommissions: '/api/account/affiliate/commissions',
    affiliateCommissionsExport: '/api/account/affiliate/commissions/export',
    passwordEmailCode: '/api/account/password/email-code',
    passwordChange: '/api/account/password/change',
    verifyEmail: '/api/account/email/verify',
    identities: '/api/account/identities',
    identity: (id: string) => `/api/account/identities/${id}`,
    oauthStart: (provider: string) => `/api/account/oauth/${provider}/start`,
    oauthCallback: (provider: string) => `/api/account/oauth/${provider}/callback`,
    walletLink: '/api/account/wallet/link',
  },
  captcha: {
    config: '/api/captcha/config',
    apiEndpoint: '/api/captcha/',
  },
  navbar: '/api/navbar',
  i18n: {
    resources: '/api/i18n/resources',
  },
  siteInfo: '/api/site-info',
  users: '/api/users',
  user: (id: string) => `/api/users/${id}`,
  userIdentity: (userId: string, identityId: string) =>
    `/api/users/${userId}/identities/${identityId}`,
  rbac: {
    roles: '/api/rbac/roles',
    role: (code: string) => `/api/rbac/roles/${code}`,
    rolePermissions: (code: string) => `/api/rbac/roles/${code}/permissions`,
    apis: '/api/rbac/apis',
    unboundApis: '/api/rbac/apis/unbound',
    api: (id: string) => `/api/rbac/apis/${id}`,
    apiMenus: (id: string) => `/api/rbac/apis/${id}/menus`,
    menuSections: '/api/rbac/menu-sections',
    menuSection: (id: string) => `/api/rbac/menu-sections/${id}`,
    menuItems: '/api/rbac/menu-items',
    menuItem: (id: string) => `/api/rbac/menu-items/${id}`,
    menuItemApis: (id: string) => `/api/rbac/menu-items/${id}/apis`,
  },
  adminModels: {
    global: '/api/admin/models/global',
    globalById: (id: string) => `/api/admin/models/global/${id}`,
    globalBatchDelete: '/api/admin/models/global/batch-delete',
    globalProviders: (id: string) => `/api/admin/models/global/${id}/providers`,
    catalog: '/api/admin/models/catalog',
    external: '/api/admin/models/external',
  },
  models: {
    catalog: '/api/models/catalog',
  },
  adminGroups: {
    list: '/api/admin/groups',
    byId: (id: string) => `/api/admin/groups/${id}`,
  },
  adminUserGroups: {
    list: '/api/admin/user-groups',
    byCode: (code: string) => `/api/admin/user-groups/${encodeURIComponent(code)}`,
    users: (code: string) => `/api/admin/user-groups/${encodeURIComponent(code)}/users`,
  },
  adminProviders: {
    list: '/api/admin/providers',
    quickImportPreview: '/api/admin/providers/quick-import/preview',
    quickImportCommit: '/api/admin/providers/quick-import/commit',
    quickImportAppendPreview: (id: string) =>
      `/api/admin/providers/${id}/quick-import/append/preview`,
    quickImportAppendCommit: (id: string) =>
      `/api/admin/providers/${id}/quick-import/append/commit`,
    quickImportBindPreview: (id: string) =>
      `/api/admin/providers/${id}/quick-import/bind/preview`,
    quickImportBindCommit: (id: string) =>
      `/api/admin/providers/${id}/quick-import/bind/commit`,
    quickImportSync: (id: string) => `/api/admin/providers/${id}/quick-import-sync`,
    byId: (id: string) => `/api/admin/providers/${id}`,
    keyGroups: '/api/admin/provider-key-groups',
    keyGroupById: (id: string) => `/api/admin/provider-key-groups/${id}`,
    cooldowns: '/api/admin/provider-cooldowns',
    releaseCooldown: (id: string) => `/api/admin/provider-cooldowns/${id}/release`,
    endpoints: (id: string) => `/api/admin/providers/${id}/endpoints`,
    endpointById: (providerId: string, endpointId: string) =>
      `/api/admin/providers/${providerId}/endpoints/${endpointId}`,
    keys: (id: string) => `/api/admin/providers/${id}/keys`,
    keyBatchPriorities: '/api/admin/providers/keys/batch-priorities',
    keyById: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}`,
    keyQuickImportResolution: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/quick-import-resolution`,
    keyQuickImportAcceptCurrent: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/quick-import-resolution/accept-current`,
    keyQuickImportRelink: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/quick-import-resolution/relink`,
    keyQuickImportModelAssociations: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/quick-import-model-associations`,
    upstreamModels: (id: string) => `/api/admin/providers/${id}/upstream-models`,
    models: (id: string) => `/api/admin/providers/${id}/models`,
    modelById: (providerId: string, modelId: string) =>
      `/api/admin/providers/${providerId}/models/${modelId}`,
    modelTest: (providerId: string, modelId: string) =>
      `/api/admin/providers/${providerId}/models/${modelId}/test`,
    modelCosts: (providerId: string) => `/api/admin/providers/${providerId}/model-costs`,
    keyModelCosts: (providerId: string, keyId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/model-costs`,
    keyModelCostByModel: (providerId: string, keyId: string, providerModelId: string) =>
      `/api/admin/providers/${providerId}/keys/${keyId}/model-costs/${providerModelId}`,
  },
  adminRequestRecords: {
    list: '/api/admin/request-records',
    active: '/api/admin/request-records/active',
    byId: (requestId: string) => `/api/admin/request-records/${requestId}`,
  },
  usageRecords: {
    list: '/api/request-records',
  },
  performanceMonitoring: {
    overview: '/api/admin/performance-monitoring/overview',
    realtime: '/api/admin/performance-monitoring/realtime',
    analytics: '/api/admin/performance-monitoring/analytics',
  },
  routing: {
    profiles: '/api/admin/routing/profiles',
    profile: (id: string) => `/api/admin/routing/profiles/${id}`,
    rankings: '/api/admin/routing/rankings',
    decision: (requestId: string) => `/api/admin/routing/decisions/${requestId}`,
  },
  modelStatus: {
    checks: '/api/model-status/checks',
  },
  adminModelStatus: {
    checks: '/api/admin/model-status/checks',
    batchCreate: '/api/admin/model-status/checks/batch-create',
    batchDelete: '/api/admin/model-status/checks/batch-delete',
    batchUpdate: '/api/admin/model-status/checks/batch-update',
    runs: '/api/admin/model-status/runs',
    byId: (id: string) => `/api/admin/model-status/checks/${id}`,
  },
  cacheMonitoring: {
    affinities: '/api/admin/monitoring/cache/affinities',
    affinityById: (affinityKey: string, endpointId: string, modelId: string, apiFormat: string) =>
      `/api/admin/monitoring/cache/affinities/${encodeURIComponent(affinityKey)}/${encodeURIComponent(endpointId)}/${encodeURIComponent(modelId)}/${encodeURIComponent(apiFormat)}`,
    clearAll: '/api/admin/monitoring/cache',
  },
  dashboard: {
    overview: '/api/dashboard/overview',
    activity: '/api/dashboard/activity',
    filterOptions: '/api/dashboard/filter-options',
    userStatsLeaderboard: '/api/admin/stats/leaderboard/users',
    apiKeyLeaderboard: '/api/admin/stats/leaderboard/api-keys',
    userUsageStats: '/api/admin/usage/stats',
    userStatsTimeSeries: '/api/admin/stats/time-series',
    costForecast: '/api/admin/stats/cost/forecast',
    costSavings: '/api/admin/stats/cost/savings',
    providerAggregation: '/api/admin/usage/aggregation/stats',
  },
  groups: {
    available: '/api/groups/available',
  },
  apiTokens: {
    list: '/api/tokens',
    byId: (id: string) => `/api/tokens/${id}`,
    secret: (id: string) => `/api/tokens/${id}/secret`,
  },
  adminApiTokens: {
    list: '/api/admin/tokens',
    byId: (id: string) => `/api/admin/tokens/${id}`,
    secret: (id: string) => `/api/admin/tokens/${id}/secret`,
  },
  adminSettings: {
    system: '/api/admin/settings/system',
    smtpTest: '/api/admin/settings/smtp/test',
  },
  adminScheduledTasks: {
    list: '/api/admin/scheduled-tasks',
    byCode: (code: string) => `/api/admin/scheduled-tasks/${code}`,
    runs: '/api/admin/scheduled-task-runs',
  },
  adminI18n: {
    languages: '/api/admin/i18n/languages',
    language: (code: string) => `/api/admin/i18n/languages/${code}`,
    translations: '/api/admin/i18n/translations',
    translation: (id: string) => `/api/admin/i18n/translations/${id}`,
    translationBundle: (namespace: string, groupKey: string, itemKey: string) =>
      `/api/admin/i18n/translations/${namespace}/${groupKey}/${itemKey}`,
  },
  announcements: {
    list: '/api/announcements',
    byId: (id: string) => `/api/announcements/${id}`,
  },
  adminAnnouncements: {
    list: '/api/admin/announcements',
    byId: (id: string) => `/api/admin/announcements/${id}`,
  },
  tickets: {
    list: '/api/tickets',
    byId: (id: string) => `/api/tickets/${id}`,
    messages: (id: string) => `/api/tickets/${id}/messages`,
  },
  adminTickets: {
    list: '/api/admin/tickets',
    byId: (id: string) => `/api/admin/tickets/${id}`,
    messages: (id: string) => `/api/admin/tickets/${id}/messages`,
  },
  notifications: {
    list: '/api/notifications',
    readAll: '/api/notifications/read-all',
    deleteRead: '/api/notifications/read',
    read: (sourceType: string, sourceId: string) =>
      `/api/notifications/${sourceType}/${sourceId}/read`,
    delete: (sourceType: string, sourceId: string) =>
      `/api/notifications/${sourceType}/${sourceId}`,
  },
  wallet: {
    balance: '/api/wallet/balance',
    transactions: '/api/wallet/transactions',
    ledgerEntries: '/api/wallet/ledger-entries',
    dailyModelUsage: '/api/wallet/ledger-entries/daily-model-usage',
  },
  adminWallets: {
    list: '/api/admin/wallets',
    ledger: '/api/admin/wallets/ledger',
    ledgerEntries: '/api/admin/wallets/ledger-entries',
    consumptionSummary: '/api/admin/wallets/ledger-consumption-summary',
    userBalance: (userId: string) => `/api/admin/wallets/users/${userId}/balance`,
    transactions: (id: string) => `/api/admin/wallets/${id}/transactions`,
    ledgerEntriesForWallet: (id: string) => `/api/admin/wallets/${id}/ledger-entries`,
    dailyModelUsageForWallet: (id: string) =>
      `/api/admin/wallets/${id}/ledger-entries/daily-model-usage`,
    adjust: (id: string) => `/api/admin/wallets/${id}/adjust`,
    recharge: (id: string) => `/api/admin/wallets/${id}/recharge`,
  },
  cardCodes: {
    redeem: '/api/card-codes/redeem',
    list: '/api/admin/card-codes',
    generate: '/api/admin/card-codes/generate',
    batchStatus: '/api/admin/card-codes/batch-status',
    types: '/api/admin/card-code-types',
    type: (id: string) => `/api/admin/card-code-types/${id}`,
  },
  adminRecharges: {
    packages: '/api/admin/recharge-packages',
    package: (id: string) => `/api/admin/recharge-packages/${id}`,
    orders: '/api/admin/recharge-orders',
    orderSummary: '/api/admin/recharge-orders/summary',
    paymentCallbacks: '/api/admin/payment-callbacks',
    paymentChannels: '/api/admin/payment-channels',
    paymentChannel: (code: string) => `/api/admin/payment-channels/${code}`,
  },
  adminAffiliates: {
    overview: '/api/admin/affiliates/overview',
    relations: '/api/admin/affiliates/relations',
    relation: (userId: string) => `/api/admin/affiliates/relations/${userId}`,
    relationChanges: '/api/admin/affiliates/relation-changes',
    commissions: '/api/admin/affiliates/commissions',
    reports: '/api/admin/affiliates/reports',
    export: '/api/admin/affiliates/reports/export',
  },
  recharges: {
    packages: '/api/recharge-packages',
    orders: '/api/recharge-orders',
    paymentChannels: '/api/payment-channels',
  },
  mail: {
    list: '/api/mail/list',
    details: '/api/mail/details',
    labels: '/api/mail/labels',
  },
  post: {
    list: '/api/post/list',
    details: '/api/post/details',
    latest: '/api/post/latest',
    search: '/api/post/search',
  },
} as const;
