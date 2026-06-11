'use client';

import type { Dispatch, ReactNode, SetStateAction } from 'react';
import type { ProviderGroup } from 'src/types/provider-group';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { type QuickImportFormState } from './provider-quick-import-utils';
import { ProviderQuickImportSyncConfigFields } from './provider-quick-import-sync-section';
import {
  DEFAULT_PROVIDER_MAX_RETRIES,
  DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS,
} from './provider-management-utils';

type Props = {
  form: QuickImportFormState;
  groups: ProviderGroup[];
  setForm: Dispatch<SetStateAction<QuickImportFormState>>;
};

export function ProviderQuickImportSourceStep({ form, groups, setForm }: Props) {
  return (
    <Stack spacing={2.75} divider={<Divider flexItem />}>
      <QuickImportSection titleKey="providers.quickImportProviderSection">
        <ProviderIdentityFields form={form} groups={groups} setForm={setForm} />
      </QuickImportSection>
      <QuickImportSection
        titleKey="providers.quickImportSourceSection"
        descriptionKey="providers.quickImportNewApiSourceDescription"
      >
        <SourceCredentialFields form={form} setForm={setForm} />
      </QuickImportSection>
      <QuickImportSection titleKey="providers.quickImportRequestSection">
        <ProviderRequestConfigFields form={form} setForm={setForm} />
      </QuickImportSection>
      <QuickImportSection titleKey="providers.quickImportSyncSection">
        <ProviderQuickImportSyncConfigFields
          form={form.sync}
          onChange={(sync) => setForm((current) => ({ ...current, sync }))}
        />
      </QuickImportSection>
      <QuickImportSection titleKey="providers.quickImportStateSection">
        <ProviderSwitchFields form={form} setForm={setForm} />
      </QuickImportSection>
    </Stack>
  );
}

function QuickImportSection({
  titleKey,
  descriptionKey,
  children,
}: {
  titleKey: string;
  descriptionKey?: string;
  children: ReactNode;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={descriptionKey ? 2 : 1.5}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle2" sx={{ color: 'text.primary' }}>
          {t(titleKey)}
        </Typography>
        {descriptionKey ? (
          <Alert severity="info" variant="outlined" sx={{ py: 0.5, mb: 0.5 }}>
            {t(descriptionKey)}
          </Alert>
        ) : null}
      </Stack>
      {children}
    </Stack>
  );
}

function ProviderIdentityFields({ form, groups, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextFieldRow
        required
        label={t('fields.providerName')}
        value={form.providerName}
        onChange={(value) => setForm((current) => ({ ...current, providerName: value }))}
      />
      <TextFieldRow
        select
        label={t('providers.providerGroup')}
        value={form.provider_group_id}
        onChange={(value) => setForm((current) => ({ ...current, provider_group_id: value }))}
      >
        <MenuItem value="">{t('providers.unclassifiedProviderGroup')}</MenuItem>
        {groups.map((group) => (
          <MenuItem key={group.id} value={group.id}>
            {group.name}
          </MenuItem>
        ))}
      </TextFieldRow>
    </Stack>
  );
}

function SourceCredentialFields({
  form,
  setForm,
}: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          select
          label={t('providers.quickImportType')}
          value="newapi"
          disabled
          sx={{ maxWidth: { md: 220 } }}
          onChange={() => undefined}
        >
          <MenuItem value="newapi">newapi</MenuItem>
        </TextFieldRow>
        <TextFieldRow
          required
          label={t('providers.quickImportBaseUrl')}
          value={form.baseUrl}
          onChange={(value) => setForm((current) => ({ ...current, baseUrl: value }))}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          type="password"
          label={t('providers.quickImportSystemToken')}
          value={form.systemAccessToken}
          onChange={(value) => setForm((current) => ({ ...current, systemAccessToken: value }))}
        />
        <TextFieldRow
          required
          label={t('providers.quickImportUserId')}
          value={form.userId}
          onChange={(value) => setForm((current) => ({ ...current, userId: value }))}
        />
        <TextFieldRow
          required
          type="number"
          label={t('providers.quickImportRechargeMultiplier')}
          value={form.rechargeMultiplier}
          onChange={(value) => setForm((current) => ({ ...current, rechargeMultiplier: value }))}
        />
      </Stack>
    </Stack>
  );
}

function ProviderRequestConfigFields({
  form,
  setForm,
}: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('providers.maxRetries')}
          value={form.max_retries}
          placeholder={String(DEFAULT_PROVIDER_MAX_RETRIES)}
          helperText={t('providers.defaultWhenBlank')}
          onChange={(value) => setForm((current) => ({ ...current, max_retries: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('providers.priority')}
          value={form.priority}
          onChange={(value) => setForm((current) => ({ ...current, priority: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('providers.requestTimeoutSeconds')}
          value={form.request_timeout_seconds}
          placeholder={String(DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS)}
          helperText={t('providers.defaultWhenBlank')}
          onChange={(value) => setForm((current) => ({ ...current, request_timeout_seconds: value }))}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('providers.streamFirstByteTimeoutSeconds')}
          value={form.stream_first_byte_timeout_seconds}
          placeholder={String(DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS)}
          helperText={t('providers.defaultWhenBlank')}
          onChange={(value) => setForm((current) => ({ ...current, stream_first_byte_timeout_seconds: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('providers.streamIdleTimeoutSeconds')}
          value={form.stream_idle_timeout_seconds}
          placeholder={String(DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS)}
          helperText={t('providers.defaultWhenBlank')}
          onChange={(value) => setForm((current) => ({ ...current, stream_idle_timeout_seconds: value }))}
        />
      </Stack>
    </Stack>
  );
}

function ProviderSwitchFields({
  form,
  setForm,
}: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <SwitchRow
        checked={form.enable_format_conversion}
        label={t('providers.enableFormatConversion')}
        onChange={(checked) => setForm((current) => ({ ...current, enable_format_conversion: checked }))}
      />
      <SwitchRow
        checked={form.keep_priority_on_conversion}
        label={t('providers.keepPriorityOnConversion')}
        onChange={(checked) => setForm((current) => ({ ...current, keep_priority_on_conversion: checked }))}
      />
      <SwitchRow
        checked={form.is_active}
        label={t('common.enabled')}
        onChange={(checked) => setForm((current) => ({ ...current, is_active: checked }))}
      />
    </Stack>
  );
}
