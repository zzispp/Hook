'use client';

import type { BillingGroup } from 'src/types/group';
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

import { useGlobalModels } from 'src/actions/models';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRoutingDecision, useRoutingProfiles, useRoutingRankings, useRoutingWindowRankings } from 'src/actions/routing';

import { AdminBreadcrumbs } from './shared';
import { RoutingDecisionDrawer } from './routing-decision-drawer';
import { RoutingHeaderActions } from './routing-observability-controls';
import { RoutingRankingPanel, RoutingConfigurationPanel } from './routing-observability-panels';

const DETAIL_WINDOWS: RoutingMetricWindow[] = ['1h', '24h', '7d'];
const DEFAULT_API_FORMAT = 'openai:chat';
const MAX_PAGE_SIZE = 100;

export function RoutingObservabilityView() {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();
  const [selectedGroup, setSelectedGroup] = useState<BillingGroup | null>(null);
  const [selectedModel, setSelectedModel] = useState<GlobalModelResponse | null>(null);
  const [groupSearch, setGroupSearch] = useState('');
  const [modelSearch, setModelSearch] = useState('');
  const [apiFormat, setApiFormat] = useState(DEFAULT_API_FORMAT);
  const [isStream, setIsStream] = useState(true);
  const [metricWindow, setMetricWindow] = useState<RoutingMetricWindow>('5m');
  const [includeExcluded, setIncludeExcluded] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [decisionRequestId, setDecisionRequestId] = useState<string | null>(null);
  const [detailItem, setDetailItem] = useState<RouteScoreExplanation | null>(null);

  const groups = useBillingGroups(0, MAX_PAGE_SIZE, {
    is_active: true,
    search: groupSearch.trim() || undefined,
  });
  const models = useGlobalModels(selectedGroup ? 0 : -1, MAX_PAGE_SIZE, {
    is_active: true,
    search: modelSearch.trim() || undefined,
  });

  const modelOptions = useMemo(() => {
    if (!selectedGroup) return [];
    return models.items.filter(
      (model) =>
        model.is_active &&
        (selectedGroup.allowed_model_ids.length === 0 || selectedGroup.allowed_model_ids.includes(model.id))
    );
  }, [models.items, selectedGroup]);

  const submittedQuery = useMemo<RoutingRankingsQuery | null>(() => {
    if (!selectedGroup || !selectedModel) return null;
    return {
      group_code: selectedGroup.code,
      model: selectedModel.name,
      api_format: apiFormat,
      is_stream: isStream,
      window: metricWindow,
      include_excluded: includeExcluded,
    };
  }, [apiFormat, includeExcluded, isStream, metricWindow, selectedGroup, selectedModel]);

  const rankings = useRoutingRankings(submittedQuery, autoRefresh);
  const detailWindows = useMemo(
    () => DETAIL_WINDOWS.filter((item) => item !== metricWindow),
    [metricWindow]
  );
  const windowRankings = useRoutingWindowRankings(submittedQuery, detailWindows, autoRefresh);
  const decision = useRoutingDecision(decisionRequestId);
  const error =
    profiles.error ??
    groups.error ??
    models.error ??
    rankings.error ??
    windowRankings.error ??
    decision.error;
  const selectedProfile = useMemo(() => {
    if (rankings.data?.profile) {
      return rankings.data.profile;
    }
    const hasProfileContext = Boolean(selectedGroup || selectedModel);
    const profileId =
      selectedModel?.routing_profile_id ??
      selectedGroup?.routing_profile_id ??
      (hasProfileContext ? 'balanced' : null);
    return profiles.items.find((item) => item.id === profileId) ?? null;
  }, [profiles.items, rankings.data?.profile, selectedGroup, selectedModel]);
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
            loading={rankings.isValidating || profiles.isValidating || groups.isValidating || models.isValidating}
            t={t}
            onAutoRefreshChange={setAutoRefresh}
            onRefresh={() => {
              void profiles.refresh();
              void groups.refresh();
              void models.refresh();
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
          groups={withSelectedOption(groups.items, selectedGroup)}
          models={withSelectedOption(modelOptions, selectedModel)}
          selectedGroup={selectedGroup}
          selectedModel={selectedModel}
          profiles={profiles.items}
          settingsLoading={groups.isLoading || models.isLoading}
          groupsLoading={groups.isLoading}
          modelsLoading={models.isLoading}
          groupSearch={groupSearch}
          modelSearch={modelSearch}
          apiFormat={apiFormat}
          isStream={isStream}
          metricWindow={metricWindow}
          includeExcluded={includeExcluded}
          onGroupSearchChange={setGroupSearch}
          onModelSearchChange={setModelSearch}
          onGroupChange={(value) => {
            setSelectedGroup(value);
            setSelectedModel(null);
            setGroupSearch(value?.name ?? value?.code ?? '');
            setModelSearch('');
            setDetailItem(null);
          }}
          onModelChange={(value) => {
            setSelectedModel(value);
            setModelSearch(value?.display_name ?? value?.name ?? '');
            setDetailItem(null);
          }}
          onApiFormatChange={setApiFormat}
          onStreamChange={setIsStream}
          onWindowChange={setMetricWindow}
          onIncludeExcludedChange={setIncludeExcluded}
          onSaved={() => {
            void groups.refresh();
            void models.refresh();
            if (submittedQuery) {
              void profiles.refresh();
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

function withSelectedOption<T extends { id: string }>(items: T[], selected: T | null) {
  if (!selected) return items;
  if (items.some((item) => item.id === selected.id)) return items;
  return [selected, ...items];
}
