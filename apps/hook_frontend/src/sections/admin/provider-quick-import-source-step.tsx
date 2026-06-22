'use client';

import type { Dispatch, ReactNode, SetStateAction } from 'react';
import type { QuickImportAuthTab } from './provider-quick-import-source';

import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { type QuickImportFormState } from './provider-quick-import-utils';
import { ProviderQuickImportSyncConfigFields } from './provider-quick-import-sync-section';
import { ProviderQuickImportSub2apiTokenHelp } from './provider-quick-import-sub2api-token-help';
import {
  DEFAULT_PROVIDER_MAX_RETRIES,
  DEFAULT_PROVIDER_REQUEST_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_IDLE_TIMEOUT_SECONDS,
  DEFAULT_PROVIDER_STREAM_FIRST_BYTE_TIMEOUT_SECONDS,
} from './provider-management-utils';

type Props = {
  form: QuickImportFormState;
  setForm: Dispatch<SetStateAction<QuickImportFormState>>;
};

export function ProviderQuickImportSourceStep({ form, setForm }: Props) {
  return (
    <Stack spacing={2.75} divider={<Divider flexItem />}>
      <QuickImportSection titleKey="providers.quickImportProviderSection">
        <ProviderIdentityFields form={form} setForm={setForm} />
      </QuickImportSection>
      <QuickImportSection
        titleKey="providers.quickImportSourceSection"
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

function ProviderIdentityFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <TextFieldRow
        required
        label={t('fields.providerName')}
        value={form.providerName}
        onChange={(value) => setForm((current) => ({ ...current, providerName: value }))}
      />
    </Stack>
  );
}

function SourceCredentialFields({
  form,
  setForm,
}: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');
  const isSub2api = form.sourceKind === 'sub2api';

  return (
    <Stack spacing={2}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          select
          label={t('providers.quickImportType')}
          value={form.sourceKind}
          sx={{ maxWidth: { md: 220 } }}
          onChange={(sourceKind) =>
            setForm((current) => ({ ...current, sourceKind: sourceKind as QuickImportFormState['sourceKind'] }))
          }
        >
          <MenuItem value="newapi">newapi</MenuItem>
          <MenuItem value="sub2api">sub2api</MenuItem>
        </TextFieldRow>
        <TextFieldRow
          required
          label={t('providers.quickImportBaseUrl')}
          value={form.baseUrl}
          onChange={(value) => setForm((current) => ({ ...current, baseUrl: value }))}
        />
      </Stack>
      {isSub2api ? (
        <Sub2apiAuthFields form={form} setForm={setForm} />
      ) : (
        <>
          <Alert severity="info" variant="outlined">
            {t('providers.quickImportNewApiSourceDescription')}
          </Alert>
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
        </>
      )}
    </Stack>
  );
}

function Sub2apiAuthFields({ form, setForm }: Pick<Props, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2}>
      <Alert severity="info" variant="outlined">
        {t('providers.quickImportSub2apiSourceDescription')}
      </Alert>
      <Tabs
        value={form.sub2apiAuthTab}
        onChange={(_event, value: QuickImportAuthTab) => setForm((current) => ({ ...current, sub2apiAuthTab: value }))}
      >
        <Tab value="password" label={t('providers.quickImportSub2apiPasswordImport')} />
        <Tab value="token" label={t('providers.quickImportSub2apiTokenImport')} />
      </Tabs>
      {form.sub2apiAuthTab === 'token' ? (
        <>
          <ProviderQuickImportSub2apiTokenHelp
            onApply={({ authToken, refreshToken, tokenExpiresAt }) =>
              setForm((current) => ({ ...current, authToken, refreshToken, tokenExpiresAt }))
            }
          />
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              required
              label={t('providers.quickImportSub2apiAuthToken')}
              value={form.authToken}
              onChange={(value) => setForm((current) => ({ ...current, authToken: value }))}
            />
            <TextFieldRow
              required
              type="password"
              label={t('providers.quickImportSub2apiRefreshToken')}
              value={form.refreshToken}
              onChange={(value) => setForm((current) => ({ ...current, refreshToken: value }))}
            />
            <TextFieldRow
              required
              label={t('providers.quickImportSub2apiTokenExpiresAt')}
              value={form.tokenExpiresAt}
              onChange={(value) => setForm((current) => ({ ...current, tokenExpiresAt: value }))}
            />
            <TextFieldRow
              required
              type="number"
              label={t('providers.quickImportRechargeMultiplier')}
              value={form.rechargeMultiplier}
              onChange={(value) => setForm((current) => ({ ...current, rechargeMultiplier: value }))}
            />
          </Stack>
        </>
      ) : (
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            required
            label={t('fields.email')}
            value={form.email}
            onChange={(value) => setForm((current) => ({ ...current, email: value }))}
          />
          <TextFieldRow
            required
            type="password"
            label={t('fields.password')}
            value={form.password}
            onChange={(value) => setForm((current) => ({ ...current, password: value }))}
          />
          <TextFieldRow
            required
            type="number"
            label={t('providers.quickImportRechargeMultiplier')}
            value={form.rechargeMultiplier}
            onChange={(value) => setForm((current) => ({ ...current, rechargeMultiplier: value }))}
          />
        </Stack>
      )}
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
