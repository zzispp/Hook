'use client';

import type {
  RoutingMetricWindow,
  RouteScoreExplanation,
  RoutingRankingResponse,
} from 'src/types/routing';

import { useMemo, useState, useEffect } from 'react';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';

import { useGlobalModels } from 'src/actions/models';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAdminApiTokens } from 'src/actions/api-tokens';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRoutingDecision, useRoutingProfiles, useRoutingRankings, useRoutingWindowRankings } from 'src/actions/routing';

import { AdminBreadcrumbs } from './shared';
import { RoutingDecisionDrawer } from './routing-decision-drawer';
import { RoutingRankingPanel, RoutingConfigurationPanel } from './routing-observability-panels';
import {
  RoutingHeaderActions,
  useDefaultRoutingFilters,
} from './routing-observability-controls';

const DETAIL_WINDOWS: RoutingMetricWindow[] = ['1h', '24h', '7d'];
const DEFAULT_API_FORMAT = 'openai:chat';
const PAGE_SIZE = 100;

export function RoutingObservabilityView() {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();
  const models = useGlobalModels(0, PAGE_SIZE, { is_active: true });
  const groups = useBillingGroups(0, PAGE_SIZE, { is_active: true });
  const apiTokens = useAdminApiTokens(0, PAGE_SIZE, { is_active: true });
  const settings = useSystemSettings();
  const [apiTokenId, setApiTokenId] = useState('');
  const [modelName, setModelName] = useState('');
  const [apiFormat, setApiFormat] = useState(DEFAULT_API_FORMAT);
  const [isStream, setIsStream] = useState(false);
  const [metricWindow, setMetricWindow] = useState<RoutingMetricWindow>('5m');
  const [includeExcluded, setIncludeExcluded] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [requestIdSeed, setRequestIdSeed] = useState(createRequestIdSeed);
  const [requestInput, setRequestInput] = useState('');
  const [decisionRequestId, setDecisionRequestId] = useState<string | null>(null);
  const [detailItem, setDetailItem] = useState<RouteScoreExplanation | null>(null);

  useDefaultRoutingFilters({
    apiTokens: apiTokens.items,
    models: models.items,
    apiTokenId,
    modelName,
    onApiTokenChange: setApiTokenId,
    onModelChange: setModelName,
  });

  const selectedToken = useMemo(
    () => apiTokens.items.find((item) => item.id === apiTokenId) ?? null,
    [apiTokenId, apiTokens.items]
  );
  const groupCode = selectedToken?.group_code ?? '';
  const selectedGroup = useMemo(
    () => groups.items.find((item) => item.code === groupCode) ?? null,
    [groupCode, groups.items]
  );
  const selectedModel = useMemo(
    () => models.items.find((item) => item.name === modelName) ?? null,
    [modelName, models.items]
  );

  useEffect(() => {
    setRequestIdSeed(createRequestIdSeed());
  }, [apiTokenId, apiFormat, isStream, modelName]);

  const query = useMemo(() => {
    if (!apiTokenId || !modelName || !apiFormat || !requestIdSeed) return null;
    return {
      api_token_id: apiTokenId,
      model: modelName,
      api_format: apiFormat,
      is_stream: isStream,
      window: metricWindow,
      include_excluded: includeExcluded,
      request_id_seed: requestIdSeed,
    };
  }, [apiFormat, apiTokenId, includeExcluded, isStream, metricWindow, modelName, requestIdSeed]);

  const rankings = useRoutingRankings(query, autoRefresh);
  const detailWindows = useMemo(
    () => DETAIL_WINDOWS.filter((item) => item !== metricWindow),
    [metricWindow]
  );
  const windowRankings = useRoutingWindowRankings(query, detailWindows, autoRefresh);
  const decision = useRoutingDecision(decisionRequestId);
  const error =
    profiles.error ??
    models.error ??
    groups.error ??
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
              void apiTokens.refresh();
              void rankings.refresh();
              void windowRankings.refresh();
            }}
          />
        }
      />

      <Stack spacing={3}>
        {error ? <Alert severity="error">{error.message}</Alert> : null}

        <RoutingConfigurationPanel
          t={t}
          apiTokens={apiTokens.items}
          models={models.items}
          selectedGroup={selectedGroup}
          selectedModel={selectedModel}
          profiles={profiles.items}
          settingsLoading={settings.isLoading || apiTokens.isLoading || groups.isLoading || models.isLoading}
          apiTokenId={apiTokenId}
          groupCode={groupCode}
          modelName={modelName}
          apiFormat={apiFormat}
          isStream={isStream}
          metricWindow={metricWindow}
          includeExcluded={includeExcluded}
          requestInput={requestInput}
          onApiTokenChange={setApiTokenId}
          onModelChange={setModelName}
          onApiFormatChange={setApiFormat}
          onStreamChange={setIsStream}
          onWindowChange={setMetricWindow}
          onIncludeExcludedChange={setIncludeExcluded}
          onRequestInputChange={setRequestInput}
          onDecisionLookup={() => {
            setDetailItem(null);
            setDecisionRequestId(requestInput.trim() || null);
          }}
          onSaved={() => {
            void groups.refresh();
            void models.refresh();
            void rankings.refresh();
            void windowRankings.refresh();
          }}
        />

        <RoutingRankingPanel
          t={t}
          selectedProfile={selectedProfile}
          rankingRows={rankings.data?.items ?? []}
          rankingsLoading={rankings.isLoading}
          onOpenRanking={(item) => {
            setDecisionRequestId(null);
            setDetailItem(item);
          }}
          onSaved={() => {
            void profiles.refresh();
            void rankings.refresh();
            void windowRankings.refresh();
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
}

function createRequestIdSeed() {
  return globalThis.crypto.randomUUID();
}
