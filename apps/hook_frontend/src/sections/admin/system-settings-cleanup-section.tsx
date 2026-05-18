import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

export function CleanupSettingsSection({
  form,
  setForm,
}: {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.cleanup')}>
      <Stack spacing={2}>
        <CleanupPanel
          title={t('systemSettings.cleanup.requestRecord.title')}
          description={t('systemSettings.cleanup.requestRecord.description')}
        >
          <SwitchRow
            checked={form.request_record_cleanup_enabled}
            label={t('systemSettings.fields.requestRecordCleanupEnabled')}
            helperText={t('systemSettings.helper.requestRecordCleanupEnabled')}
            onChange={(checked) =>
              setForm((current) => ({ ...current, request_record_cleanup_enabled: checked }))
            }
          />
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              type="number"
              label={t('systemSettings.fields.requestRecordCleanupIntervalHours')}
              value={form.request_record_cleanup_interval_hours}
              helperText={t('systemSettings.helper.requestRecordCleanupIntervalHours')}
              onChange={(value) =>
                setForm((current) => ({
                  ...current,
                  request_record_cleanup_interval_hours: value,
                }))
              }
            />
            <TextFieldRow
              type="number"
              label={t('systemSettings.fields.requestRecordRetentionDays')}
              value={form.request_record_retention_days}
              helperText={t('systemSettings.helper.requestRecordRetentionDays')}
              onChange={(value) =>
                setForm((current) => ({ ...current, request_record_retention_days: value }))
              }
            />
            <TextFieldRow
              type="number"
              label={t('systemSettings.fields.requestRecordPayloadRetentionDays')}
              value={form.request_record_payload_retention_days}
              helperText={t('systemSettings.helper.requestRecordPayloadRetentionDays')}
              onChange={(value) =>
                setForm((current) => ({
                  ...current,
                  request_record_payload_retention_days: value,
                }))
              }
            />
          </Stack>
        </CleanupPanel>

        <CleanupPanel
          title={t('systemSettings.cleanup.performanceMonitoring.title')}
          description={t('systemSettings.cleanup.performanceMonitoring.description')}
        >
          <SwitchRow
            checked={form.performance_monitoring_cleanup_enabled}
            label={t('systemSettings.fields.performanceMonitoringCleanupEnabled')}
            helperText={t('systemSettings.helper.performanceMonitoringCleanupEnabled')}
            onChange={(checked) =>
              setForm((current) => ({
                ...current,
                performance_monitoring_cleanup_enabled: checked,
              }))
            }
          />
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              type="number"
              label={t('systemSettings.fields.performanceMonitoringCleanupIntervalHours')}
              value={form.performance_monitoring_cleanup_interval_hours}
              helperText={t('systemSettings.helper.performanceMonitoringCleanupIntervalHours')}
              onChange={(value) =>
                setForm((current) => ({
                  ...current,
                  performance_monitoring_cleanup_interval_hours: value,
                }))
              }
            />
            <TextFieldRow
              type="number"
              label={t('systemSettings.fields.performanceMonitoringRetentionDays')}
              value={form.performance_monitoring_retention_days}
              helperText={t('systemSettings.helper.performanceMonitoringRetentionDays')}
              onChange={(value) =>
                setForm((current) => ({
                  ...current,
                  performance_monitoring_retention_days: value,
                }))
              }
            />
          </Stack>
        </CleanupPanel>
      </Stack>
    </SettingsSection>
  );
}

function CleanupPanel({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <Paper variant="outlined" sx={{ p: 2, borderRadius: 1 }}>
      <Stack spacing={2}>
        <Stack spacing={0.5}>
          <Typography variant="subtitle2">{title}</Typography>
          <Typography variant="body2" color="text.secondary">
            {description}
          </Typography>
        </Stack>
        {children}
      </Stack>
    </Paper>
  );
}
