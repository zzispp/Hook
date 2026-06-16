'use client';

import type {
  RoutingMetricWindow,
  RoutingRankingsQuery,
  RouteScoreExplanation,
  RoutingRankingResponse,
} from 'src/types/routing';

import { useMemo, useState, useEffect } from 'react';

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
const PAGE_SIZE = 1000;

export function RoutingObservabilityView() {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();
  const models = useGlobalModels(0, PAGE_SIZE, { is_active: true });
  const groups = useBillingGroups(0, PAGE_SIZE, { is_active: true });
  const users = useUsers(0, PAGE_SIZE, { is_active: true });
  const [userId, setUserId] = useState('');
  const apiTokens = useAdminApiTokens(userId ? 0 : -1, PAGE_SIZE, {
    is_active: true,
    user_id: userId || undefined,
  });
  const settings = useSystemSettings();
  const [apiTokenId, setApiTokenId] = useState('');
  const [modelName, setModelName] = useState('');
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

  const selectedUser = useMemo(() => users.items.find((item) => item.id === userId) ?? null, [userId, users.items]);
  const selectedToken = useMemo(
    () => apiTokens.items.find((item) => item.id === apiTokenId) ?? null,
    [apiTokenId, apiTokens.items]
  );
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
  const selectedModel = useMemo(
    () => tokenModels.find((item) => item.name === modelName) ?? null,
    [modelName, tokenModels]
  );

  useEffect(() => {
    if (modelName && !tokenModels.some((model) => model.name === modelName)) setModelName('');
  }, [modelName, tokenModels]);

  const canSimulate = Boolean(selectedToken && selectedModel && apiFormat && requestIdSeed);

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
          users={users.items}
          apiTokens={apiTokens.items}
          models={tokenModels}
          selectedGroup={selectedGroup}
          selectedModel={selectedModel}
          profiles={profiles.items}
          settingsLoading={settings.isLoading || apiTokens.isLoading || groups.isLoading || models.isLoading || users.isLoading}
          userId={userId}
          apiTokenId={apiTokenId}
          groupCode={groupCode}
          modelName={modelName}
          apiFormat={apiFormat}
          isStream={isStream}
          metricWindow={metricWindow}
          includeExcluded={includeExcluded}
          requestInput={requestInput}
          canSimulate={canSimulate}
          onUserChange={(value) => {
            setUserId(value);
            setApiTokenId('');
            setModelName('');
            clearSubmittedQuery(false);
          }}
          onApiTokenChange={(value) => {
            setApiTokenId(value);
            setModelName('');
            clearSubmittedQuery(true);
          }}
          onModelChange={(value) => {
            setModelName(value);
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
            if (!canSimulate) return;
            setSubmittedQuery({
              api_token_id: apiTokenId,
              model: modelName,
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
