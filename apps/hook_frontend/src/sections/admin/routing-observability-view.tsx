'use client';

import type {
  RoutingProfileId,
  RoutingMetricWindow,
  RouteScoreExplanation,
  RoutingRankingResponse,
} from 'src/types/routing';

import { useMemo, useState, useEffect } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useGlobalModels } from 'src/actions/models';
import { useBillingGroups } from 'src/actions/groups';
import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useSystemSettings } from 'src/actions/system-settings';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useRoutingDecision, useRoutingProfiles, useRoutingRankings, useRoutingWindowRankings } from 'src/actions/routing';

import { Iconify } from 'src/components/iconify';

import { RefreshButton, AdminBreadcrumbs } from './shared';
import { RoutingRankingTable } from './routing-ranking-table';
import { RoutingProfileEditor } from './routing-profile-editor';
import { RoutingDecisionDrawer } from './routing-decision-drawer';
import { RoutingProfileSummary } from './routing-profile-summary';
import { RoutingProfileSettings } from './routing-profile-settings';
import { formatApiFormat, API_FORMAT_OPTIONS } from './provider-management-utils';

const WINDOWS: RoutingMetricWindow[] = ['1m', '5m', '15m', '1h', '24h', '7d'];
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

  useDefaultFilters({
    groups: groups.items,
    models: models.items,
    groupCode,
    modelName,
    onGroupChange: setGroupCode,
    onModelChange: setModelName,
  });

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
          <HeaderActions
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
        <Card>
          <Stack spacing={2.5} sx={{ p: 2.5 }}>
            <RoutingProfileSettings
              group={groups.items.find((item) => item.code === groupCode) ?? null}
              model={models.items.find((item) => item.name === modelName) ?? null}
              profiles={profiles.items}
              loading={settings.isLoading || groups.isLoading || models.isLoading}
              onSaved={() => {
                void groups.refresh();
                void models.refresh();
                void rankings.refresh();
                void windowRankings.refresh();
              }}
            />
            <RoutingFilters
              t={t}
              groups={groups.items}
              models={models.items}
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
            />
            <RoutingProfileSummary profile={selectedProfile} t={t} />
            <RoutingProfileEditor
              profile={selectedProfile}
              onSaved={() => {
                void profiles.refresh();
                void rankings.refresh();
                void windowRankings.refresh();
              }}
            />
            <Tabs
              value={profileId}
              variant="scrollable"
              onChange={(_, value) => setProfileId(value as RoutingProfileId)}
            >
              {profiles.items.map((profile) => (
                <Tab key={profile.id} value={profile.id} label={profile.name} />
              ))}
            </Tabs>
            <RoutingRankingTable
              rows={rankings.data?.items ?? []}
              loading={rankings.isLoading}
              t={t}
              onOpen={(item) => {
                setDecisionRequestId(null);
                setDetailItem(item);
              }}
            />
          </Stack>
        </Card>
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

function RoutingFilters(props: FilterProps) {
  return (
    <Stack direction={{ xs: 'column', lg: 'row' }} spacing={2} alignItems={{ xs: 'stretch', lg: 'center' }}>
      <TextField select size="small" label={props.t('routing.filters.group')} value={props.groupCode} onChange={(event) => props.onGroupChange(event.target.value)} sx={{ minWidth: 160 }}>
        {props.groups.map((group) => <MenuItem key={group.code} value={group.code}>{group.name || group.code}</MenuItem>)}
      </TextField>
      <TextField select size="small" label={props.t('routing.filters.model')} value={props.modelName} onChange={(event) => props.onModelChange(event.target.value)} sx={{ minWidth: 220 }}>
        {props.models.map((model) => <MenuItem key={model.id} value={model.name}>{model.name}</MenuItem>)}
      </TextField>
      <TextField select size="small" label={props.t('routing.filters.apiFormat')} value={props.apiFormat} onChange={(event) => props.onApiFormatChange(event.target.value)} sx={{ minWidth: 180 }}>
        {API_FORMAT_OPTIONS.map((format) => <MenuItem key={format} value={format}>{formatApiFormat(format)}</MenuItem>)}
      </TextField>
      <TextField select size="small" label={props.t('routing.filters.window')} value={props.metricWindow} onChange={(event) => props.onWindowChange(event.target.value as RoutingMetricWindow)} sx={{ width: 120 }}>
        {WINDOWS.map((item) => <MenuItem key={item} value={item}>{item}</MenuItem>)}
      </TextField>
      <FormControlLabel control={<Switch checked={props.isStream} onChange={(event) => props.onStreamChange(event.target.checked)} />} label={props.t('routing.filters.stream')} />
      <FormControlLabel control={<Switch checked={props.includeExcluded} onChange={(event) => props.onIncludeExcludedChange(event.target.checked)} />} label={props.t('routing.filters.includeExcluded')} />
      <TextField size="small" value={props.requestInput} label={props.t('routing.filters.requestId')} onChange={(event) => props.onRequestInputChange(event.target.value)} sx={{ minWidth: 220 }} />
      <Button variant="outlined" color="inherit" startIcon={<Iconify icon="eva:search-fill" />} disabled={!props.requestInput.trim()} onClick={props.onDecisionLookup}>
        {props.t('routing.actions.lookup')}
      </Button>
    </Stack>
  );
}

function HeaderActions(props: {
  autoRefresh: boolean;
  loading: boolean;
  t: ReturnType<typeof useTranslate>['t'];
  onRefresh: VoidFunction;
  onAutoRefreshChange: (value: boolean) => void;
}) {
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <FormControlLabel
        control={<Switch checked={props.autoRefresh} onChange={(event) => props.onAutoRefreshChange(event.target.checked)} />}
        label={props.t('routing.filters.autoRefresh')}
      />
      <RefreshButton loading={props.loading} onClick={props.onRefresh} />
    </Stack>
  );
}

type FilterProps = {
  t: ReturnType<typeof useTranslate>['t'];
  groups: ReturnType<typeof useBillingGroups>['items'];
  models: ReturnType<typeof useGlobalModels>['items'];
  groupCode: string;
  modelName: string;
  apiFormat: string;
  isStream: boolean;
  metricWindow: RoutingMetricWindow;
  includeExcluded: boolean;
  requestInput: string;
  onGroupChange: (value: string) => void;
  onModelChange: (value: string) => void;
  onApiFormatChange: (value: string) => void;
  onStreamChange: (value: boolean) => void;
  onWindowChange: (value: RoutingMetricWindow) => void;
  onIncludeExcludedChange: (value: boolean) => void;
  onRequestInputChange: (value: string) => void;
  onDecisionLookup: VoidFunction;
};

function useDefaultFilters(input: {
  groups: ReturnType<typeof useBillingGroups>['items'];
  models: ReturnType<typeof useGlobalModels>['items'];
  groupCode: string;
  modelName: string;
  onGroupChange: (value: string) => void;
  onModelChange: (value: string) => void;
}) {
  const { groupCode, groups, modelName, models, onGroupChange, onModelChange } = input;

  useEffect(() => {
    if (!groupCode && groups[0]) onGroupChange(groups[0].code);
  }, [groupCode, groups, onGroupChange]);

  useEffect(() => {
    if (!modelName && models[0]) onModelChange(models[0].name);
  }, [modelName, models, onModelChange]);
}
