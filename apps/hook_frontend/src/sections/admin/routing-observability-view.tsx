'use client';

import type {
  RoutingProfileId,
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
const DEFAULT_PROFILE: RoutingProfileId = 'balanced';
const DEFAULT_API_FORMAT = 'openai:chat';
const PAGE_SIZE = 100;

export function RoutingObservabilityView() {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();
  const models = useGlobalModels(0, PAGE_SIZE, { is_active: true });
  const groups = useBillingGroups(0, PAGE_SIZE, { is_active: true });
  const settings = useSystemSettings();
  const [profileId, setProfileId] = useState<RoutingProfileId>(DEFAULT_PROFILE);
  const [groupCode, setGroupCode] = useState('');
  const [modelName, setModelName] = useState('');
  const [apiFormat, setApiFormat] = useState(DEFAULT_API_FORMAT);
  const [isStream, setIsStream] = useState(false);
  const [metricWindow, setMetricWindow] = useState<RoutingMetricWindow>('5m');
  const [includeExcluded, setIncludeExcluded] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [requestInput, setRequestInput] = useState('');
  const [decisionRequestId, setDecisionRequestId] = useState<string | null>(null);
  const [detailItem, setDetailItem] = useState<RouteScoreExplanation | null>(null);

  useDefaultRoutingFilters({
    groups: groups.items,
    models: models.items,
    groupCode,
    modelName,
    onGroupChange: setGroupCode,
    onModelChange: setModelName,
  });

  const selectedGroup = useMemo(
    () => groups.items.find((item) => item.code === groupCode) ?? null,
    [groupCode, groups.items]
  );
  const selectedModel = useMemo(
    () => models.items.find((item) => item.name === modelName) ?? null,
    [modelName, models.items]
  );

  const effectiveProfileId = useMemo(
    () => selectedModel?.routing_profile_id ?? selectedGroup?.routing_profile_id ?? DEFAULT_PROFILE,
    [selectedGroup?.routing_profile_id, selectedModel?.routing_profile_id]
  );

  useEffect(() => {
    if (effectiveProfileId && profiles.items.some((item) => item.id === effectiveProfileId)) {
      setProfileId(effectiveProfileId);
    }
  }, [effectiveProfileId, profiles.items]);

  useEffect(() => {
    if (profiles.items.length > 0 && !profiles.items.some((item) => item.id === profileId)) {
      setProfileId(profiles.items[0].id);
    }
  }, [profileId, profiles.items]);

  const query = useMemo(() => {
    if (!groupCode || !modelName || !apiFormat) return null;
    return {
      profile_id: profileId,
      group_code: groupCode,
      model: modelName,
      api_format: apiFormat,
      is_stream: isStream,
      window: metricWindow,
      include_excluded: includeExcluded,
    };
  }, [apiFormat, groupCode, includeExcluded, isStream, metricWindow, modelName, profileId]);

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
    settings.error ??
    rankings.error ??
    windowRankings.error ??
    decision.error;
  const selectedProfile = useMemo(
    () => profiles.items.find((item) => item.id === profileId) ?? null,
    [profileId, profiles.items]
  );
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
            loading={rankings.isValidating || profiles.isValidating}
            t={t}
            onAutoRefreshChange={setAutoRefresh}
            onRefresh={() => {
              void profiles.refresh();
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
          groups={groups.items}
          models={models.items}
          selectedGroup={selectedGroup}
          selectedModel={selectedModel}
          profiles={profiles.items}
          settingsLoading={settings.isLoading || groups.isLoading || models.isLoading}
          groupCode={groupCode}
          modelName={modelName}
          apiFormat={apiFormat}
          isStream={isStream}
          metricWindow={metricWindow}
          includeExcluded={includeExcluded}
          requestInput={requestInput}
          onGroupChange={setGroupCode}
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
          profiles={profiles.items}
          profileId={profileId}
          selectedProfile={selectedProfile}
          rankingRows={rankings.data?.items ?? []}
          rankingsLoading={rankings.isLoading}
          onProfileChange={setProfileId}
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
