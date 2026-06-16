'use client';

import type { SystemUser } from 'src/types/rbac';
import type { ApiToken } from 'src/types/api-token';
import type { GlobalModelResponse } from 'src/types/model';
import type {
  RoutingMetricWindow,
  RoutingRankingsQuery,
  RouteScoreExplanation,
  RoutingRankingResponse,
} from 'src/types/routing';

import { useMemo, useState } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';

import { useUsers } from 'src/actions/rbac';
import { useGlobalModels } from 'src/actions/models';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAdminApiTokens } from 'src/actions/api-tokens';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRoutingDecision, useRoutingProfiles, useRoutingRankings, useRoutingWindowRankings } from 'src/actions/routing';

import { AdminBreadcrumbs } from './shared';
import { routingModelsForToken } from './routing-token-access';
import { RoutingDecisionDrawer } from './routing-decision-drawer';
import { RoutingHeaderActions } from './routing-observability-controls';
import { RoutingRankingPanel, RoutingConfigurationPanel } from './routing-observability-panels';

const DETAIL_WINDOWS: RoutingMetricWindow[] = ['1h', '24h', '7d'];
const DEFAULT_API_FORMAT = 'openai:chat';
const MAX_PAGE_SIZE = 100;

export function RoutingObservabilityView() {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();
  const [selectedUser, setSelectedUser] = useState<SystemUser | null>(null);
  const [selectedToken, setSelectedToken] = useState<ApiToken | null>(null);
  const [selectedModel, setSelectedModel] = useState<GlobalModelResponse | null>(null);
  const [userSearch, setUserSearch] = useState('');
  const [apiTokenSearch, setApiTokenSearch] = useState('');
  const [modelSearch, setModelSearch] = useState('');
  const users = useUsers(0, MAX_PAGE_SIZE, {
    is_active: true,
    search: userSearch.trim() || undefined,
  });
  const apiTokens = useAdminApiTokens(selectedUser ? 0 : -1, MAX_PAGE_SIZE, {
    is_active: true,
    user_id: selectedUser?.id,
    search: apiTokenSearch.trim() || undefined,
  });
  const groups = useBillingGroups(selectedToken ? 0 : -1, MAX_PAGE_SIZE, {
    is_active: true,
    search: selectedToken?.group_code,
  });
  const models = useGlobalModels(selectedToken ? 0 : -1, MAX_PAGE_SIZE, {
    is_active: true,
    search: modelSearch.trim() || undefined,
  });
  const settings = useSystemSettings();
  const [apiFormat, setApiFormat] = useState(DEFAULT_API_FORMAT);
  const [isStream, setIsStream] = useState(false);
  const [metricWindow, setMetricWindow] = useState<RoutingMetricWindow>('5m');
  const [includeExcluded, setIncludeExcluded] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [requestIdSeed, setRequestIdSeed] = useState(createRequestIdSeed);
  const [submittedQuery, setSubmittedQuery] = useState<RoutingRankingsQuery | null>(null);
  const [requestInput, setRequestInput] = useState('');
  const [decisionRequestId, setDecisionRequestId] = useState<string | null>(null);
  const [detailItem, setDetailItem] = useState<RouteScoreExplanation | null>(null);

  const groupCode = selectedToken?.group_code ?? '';
  const selectedGroup = useMemo(
    () => groups.items.find((item) => item.code === groupCode) ?? null,
    [groupCode, groups.items]
  );
  const tokenModels = useMemo(
    () =>
      routingModelsForToken({
        token: selectedToken,
        group: selectedGroup,
        user: selectedUser,
        models: models.items,
      }),
    [models.items, selectedGroup, selectedToken, selectedUser]
  );
  const userOptions = useMemo(() => withSelectedOption(users.items, selectedUser), [selectedUser, users.items]);
  const tokenOptions = useMemo(() => withSelectedOption(apiTokens.items, selectedToken), [apiTokens.items, selectedToken]);
  const modelOptions = useMemo(() => withSelectedOption(tokenModels, selectedModel), [selectedModel, tokenModels]);

  const canSimulate = Boolean(selectedToken && selectedModel && selectedGroup && apiFormat && requestIdSeed);

  const rankings = useRoutingRankings(submittedQuery, autoRefresh);
  const detailWindows = useMemo(
    () => DETAIL_WINDOWS.filter((item) => item !== metricWindow),
    [metricWindow]
  );
  const windowRankings = useRoutingWindowRankings(submittedQuery, detailWindows, autoRefresh);
  const decision = useRoutingDecision(decisionRequestId);
  const error =
    profiles.error ??
    models.error ??
    groups.error ??
    users.error ??
    apiTokens.error ??
    settings.error ??
    rankings.error ??
    windowRankings.error ??
    decision.error;
  const selectedProfile = rankings.data?.profile ?? null;
  const drawerProfile = useMemo(
    () =>
      (decision.data
        ? profiles.items.find((item) => item.id === decision.data?.profile_id)
        : selectedProfile) || selectedProfile,
    [decision.data, profiles.items, selectedProfile]
  );
  const windowSnapshots = useMemo(() => {
    const snapshots: Partial<Record<RoutingMetricWindow, RoutingRankingResponse>> = {};
    if (rankings.data) snapshots[metricWindow] = rankings.data;
    return { ...snapshots, ...(windowRankings.data || {}) };
  }, [metricWindow, rankings.data, windowRankings.data]);

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.routing}
        action={
          <RoutingHeaderActions
            autoRefresh={autoRefresh}
            loading={rankings.isValidating || profiles.isValidating || apiTokens.isValidating}
            t={t}
            onAutoRefreshChange={setAutoRefresh}
            onRefresh={() => {
              void profiles.refresh();
              void users.refresh();
              void apiTokens.refresh();
              if (submittedQuery) {
                void rankings.refresh();
                void windowRankings.refresh();
              }
            }}
          />
        }
      />

      <Stack spacing={3}>
        {error ? <Alert severity="error">{error.message}</Alert> : null}

        <RoutingConfigurationPanel
          t={t}
          users={userOptions}
          apiTokens={tokenOptions}
          models={modelOptions}
          selectedGroup={selectedGroup}
          selectedModel={selectedModel}
          profiles={profiles.items}
          settingsLoading={settings.isLoading || apiTokens.isLoading || groups.isLoading || models.isLoading || users.isLoading}
          usersLoading={users.isLoading}
          apiTokensLoading={apiTokens.isLoading}
          modelsLoading={models.isLoading || groups.isLoading}
          selectedUser={selectedUser}
          selectedToken={selectedToken}
          groupCode={groupCode}
          userSearch={userSearch}
          apiTokenSearch={apiTokenSearch}
          modelSearch={modelSearch}
          apiFormat={apiFormat}
          isStream={isStream}
          metricWindow={metricWindow}
          includeExcluded={includeExcluded}
          requestInput={requestInput}
          canSimulate={canSimulate}
          onUserSearchChange={setUserSearch}
          onApiTokenSearchChange={setApiTokenSearch}
          onModelSearchChange={setModelSearch}
          onUserChange={(value) => {
            setSelectedUser(value);
            setSelectedToken(null);
            setSelectedModel(null);
            setUserSearch(value?.username ?? '');
            setApiTokenSearch('');
            setModelSearch('');
            clearSubmittedQuery(false);
          }}
          onApiTokenChange={(value) => {
            setSelectedToken(value);
            setSelectedModel(null);
            setApiTokenSearch(value?.name ?? '');
            setModelSearch('');
            clearSubmittedQuery(true);
          }}
          onModelChange={(value) => {
            setSelectedModel(value);
            setModelSearch(value?.display_name ?? value?.name ?? '');
            clearSubmittedQuery(true);
          }}
          onApiFormatChange={(value) => {
            setApiFormat(value);
            clearSubmittedQuery(true);
          }}
          onStreamChange={(value) => {
            setIsStream(value);
            clearSubmittedQuery(true);
          }}
          onWindowChange={(value) => {
            setMetricWindow(value);
            clearSubmittedQuery(false);
          }}
          onIncludeExcludedChange={(value) => {
            setIncludeExcluded(value);
            clearSubmittedQuery(false);
          }}
          onRequestInputChange={setRequestInput}
          onSimulate={() => {
            const token = selectedToken;
            const model = selectedModel;
            if (!canSimulate || !token || !model) return;
            setSubmittedQuery({
              api_token_id: token.id,
              model: model.name,
              api_format: apiFormat,
              is_stream: isStream,
              window: metricWindow,
              include_excluded: includeExcluded,
              request_id_seed: requestIdSeed,
            });
          }}
          onDecisionLookup={() => {
            setDetailItem(null);
            setDecisionRequestId(requestInput.trim() || null);
          }}
          onSaved={() => {
            void groups.refresh();
            void models.refresh();
            if (submittedQuery) {
              void rankings.refresh();
              void windowRankings.refresh();
            }
          }}
        />

        <RoutingRankingPanel
          t={t}
          selectedProfile={selectedProfile}
          rankingRows={rankings.data?.items ?? []}
          rankingsLoading={rankings.isLoading}
          hasSubmittedQuery={Boolean(submittedQuery)}
          onOpenRanking={(item) => {
            setDecisionRequestId(null);
            setDetailItem(item);
          }}
          onSaved={() => {
            void profiles.refresh();
            if (submittedQuery) {
              void rankings.refresh();
              void windowRankings.refresh();
            }
          }}
        />
      </Stack>

      <RoutingDecisionDrawer
        open={Boolean(detailItem || decision.data)}
        item={detailItem}
        decision={detailItem ? null : decision.data}
        profile={drawerProfile}
        windowSnapshots={windowSnapshots}
        selectedWindow={metricWindow}
        t={t}
        onClose={() => {
          setDetailItem(null);
          setDecisionRequestId(null);
        }}
      />
    </DashboardContent>
  );

  function clearSubmittedQuery(resetSeed: boolean) {
    setSubmittedQuery(null);
    setDetailItem(null);
    if (resetSeed) setRequestIdSeed(createRequestIdSeed());
  }
}

function createRequestIdSeed() {
  return globalThis.crypto.randomUUID();
}

function withSelectedOption<T extends { id: string }>(items: T[], selected: T | null) {
  if (!selected) return items;
  if (items.some((item) => item.id === selected.id)) return items;
  return [selected, ...items];
}
