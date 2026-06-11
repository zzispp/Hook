'use client';

import type { QuickImportSyncConfigForm } from './provider-quick-import-utils';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';

type Props = {
  form: QuickImportSyncConfigForm;
  disabled?: boolean;
  onChange: (form: QuickImportSyncConfigForm) => void;
};

export function ProviderQuickImportSyncConfigFields({ form, disabled, onChange }: Props) {
  const { t } = useTranslate('admin');
  const thresholdDisabled = disabled || form.fetch_failure_action !== 'disable_after_failures';

  return (
    <Stack spacing={2}>
      <SwitchRow
        checked={form.auto_sync_enabled}
        disabled={disabled}
        label={t('providers.quickImportAutoSyncEnabled')}
        onChange={(checked) => onChange({ ...form, auto_sync_enabled: checked })}
      />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          select
          disabled={disabled}
          label={t('providers.quickImportCostSyncMode')}
          value={form.cost_sync_mode}
          onChange={(value) =>
            onChange({ ...form, cost_sync_mode: value as QuickImportSyncConfigForm['cost_sync_mode'] })
          }
        >
          <MenuItem value="overwrite">{t('providers.quickImportCostSyncOverwrite')}</MenuItem>
          <MenuItem value="report_only">{t('providers.quickImportCostSyncReportOnly')}</MenuItem>
        </TextFieldRow>
        <AnomalyActionField
          disabled={disabled}
          label={t('providers.quickImportTokenDeletedAction')}
          value={form.anomaly_actions.token_deleted}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, token_deleted: value },
            })
          }
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <AnomalyActionField
          disabled={disabled}
          label={t('providers.quickImportTokenDisabledAction')}
          value={form.anomaly_actions.token_disabled}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, token_disabled: value },
            })
          }
        />
        <AnomalyActionField
          disabled={disabled}
          label={t('providers.quickImportGroupRemovedAction')}
          value={form.anomaly_actions.group_removed}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, group_removed: value },
            })
          }
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <GroupChangedActionField
          disabled={disabled}
          value={form.anomaly_actions.group_changed}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, group_changed: value },
            })
          }
        />
        <AnomalyActionField
          disabled={disabled}
          label={t('providers.quickImportKeyUnavailableAction')}
          value={form.anomaly_actions.key_unavailable}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, key_unavailable: value },
            })
          }
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <AnomalyActionField
          disabled={disabled}
          label={t('providers.quickImportModelRemovedAction')}
          value={form.anomaly_actions.model_removed}
          onChange={(value) =>
            onChange({
              ...form,
              anomaly_actions: { ...form.anomaly_actions, model_removed: value },
            })
          }
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          select
          disabled={disabled}
          label={t('providers.quickImportFetchFailureAction')}
          value={form.fetch_failure_action}
          onChange={(value) =>
            onChange({
              ...form,
              fetch_failure_action: value as QuickImportSyncConfigForm['fetch_failure_action'],
            })
          }
        >
          <MenuItem value="report_only">{t('providers.quickImportFetchFailureReportOnly')}</MenuItem>
          <MenuItem value="disable_after_failures">
            {t('providers.quickImportFetchFailureDisableAfterFailures')}
          </MenuItem>
        </TextFieldRow>
        <TextFieldRow
          type="number"
          disabled={thresholdDisabled}
          label={t('providers.quickImportFetchFailureThreshold')}
          value={form.fetch_failure_disable_threshold}
          onChange={(value) => onChange({ ...form, fetch_failure_disable_threshold: value })}
        />
      </Stack>
    </Stack>
  );
}

function AnomalyActionField({
  disabled,
  label,
  value,
  onChange,
}: {
  disabled?: boolean;
  label: string;
  value: QuickImportSyncConfigForm['anomaly_actions']['token_deleted'];
  onChange: (value: QuickImportSyncConfigForm['anomaly_actions']['token_deleted']) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      disabled={disabled}
      label={label}
      value={value}
      onChange={(next) => onChange(next as QuickImportSyncConfigForm['anomaly_actions']['token_deleted'])}
    >
      <MenuItem value="disable_key">{t('providers.quickImportAnomalyDisableKey')}</MenuItem>
      <MenuItem value="report_only">{t('providers.quickImportAnomalyReportOnly')}</MenuItem>
    </TextFieldRow>
  );
}

function GroupChangedActionField({
  disabled,
  value,
  onChange,
}: {
  disabled?: boolean;
  value: QuickImportSyncConfigForm['anomaly_actions']['group_changed'];
  onChange: (value: QuickImportSyncConfigForm['anomaly_actions']['group_changed']) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      disabled={disabled}
      label={t('providers.quickImportGroupChangedAction')}
      value={value}
      onChange={(next) => onChange(next as QuickImportSyncConfigForm['anomaly_actions']['group_changed'])}
    >
      <MenuItem value="disable_key">{t('providers.quickImportAnomalyDisableKey')}</MenuItem>
      <MenuItem value="report_only">{t('providers.quickImportAnomalyReportOnly')}</MenuItem>
      <MenuItem value="sync">{t('providers.quickImportAnomalySync')}</MenuItem>
    </TextFieldRow>
  );
}
