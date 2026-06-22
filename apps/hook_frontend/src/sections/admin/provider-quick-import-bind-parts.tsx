'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { QuickImportAuthTab } from './provider-quick-import-source';
import type { ProviderQuickImportTokenPreview, ProviderQuickImportBindPreviewResponse } from 'src/types/provider-quick-import';

import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow } from './shared';
import { ProviderQuickImportSyncConfigFields } from './provider-quick-import-sync-section';
import { ProviderQuickImportSub2apiTokenHelp } from './provider-quick-import-sub2api-token-help';
import {
  validSyncConfig,
  type QuickImportFormState,
  type QuickImportTokenDraft,
} from './provider-quick-import-utils';

type BindSummary = {
  reused: number;
  created: number;
  deleted: number;
};

const CREATE_LOCAL_KEY_VALUE = '__hook_create_local_key__';

export function BindSourceStep({
  form,
  setForm,
}: {
  form: QuickImportFormState;
  setForm: Dispatch<SetStateAction<QuickImportFormState>>;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2.5}>
      <Alert severity="warning" variant="outlined">
        {t('providers.quickImportBindDestructiveHint')}
      </Alert>
      <Stack spacing={2}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextFieldRow
            select
            label={t('providers.quickImportType')}
            value={form.sourceKind}
            onChange={(sourceKind) => setForm((current) => ({ ...current, sourceKind: sourceKind as QuickImportFormState['sourceKind'] }))}
          >
            <MenuItem value="newapi">newapi</MenuItem>
            <MenuItem value="sub2api">sub2api</MenuItem>
          </TextFieldRow>
          <TextFieldRow
            required
            label={t('providers.quickImportBaseUrl')}
            value={form.baseUrl}
            onChange={(baseUrl) => setForm((current) => ({ ...current, baseUrl }))}
          />
        </Stack>
        {form.sourceKind === 'sub2api' ? (
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
                    onChange={(authToken) => setForm((current) => ({ ...current, authToken }))}
                  />
                  <TextFieldRow
                    required
                    type="password"
                    label={t('providers.quickImportSub2apiRefreshToken')}
                    value={form.refreshToken}
                    onChange={(refreshToken) => setForm((current) => ({ ...current, refreshToken }))}
                  />
                  <TextFieldRow
                    required
                    label={t('providers.quickImportSub2apiTokenExpiresAt')}
                    value={form.tokenExpiresAt}
                    onChange={(tokenExpiresAt) => setForm((current) => ({ ...current, tokenExpiresAt }))}
                  />
                  <TextFieldRow
                    required
                    type="number"
                    label={t('providers.quickImportRechargeMultiplier')}
                    value={form.rechargeMultiplier}
                    onChange={(rechargeMultiplier) => setForm((current) => ({ ...current, rechargeMultiplier }))}
                  />
                </Stack>
              </>
            ) : (
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
                <TextFieldRow
                  required
                  label={t('fields.email')}
                  value={form.email}
                  onChange={(email) => setForm((current) => ({ ...current, email }))}
                />
                <TextFieldRow
                  required
                  type="password"
                  label={t('fields.password')}
                  value={form.password}
                  onChange={(password) => setForm((current) => ({ ...current, password }))}
                />
                <TextFieldRow
                  required
                  type="number"
                  label={t('providers.quickImportRechargeMultiplier')}
                  value={form.rechargeMultiplier}
                  onChange={(rechargeMultiplier) => setForm((current) => ({ ...current, rechargeMultiplier }))}
                />
              </Stack>
            )}
          </Stack>
        ) : (
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextFieldRow
              required
              type="password"
              label={t('providers.quickImportSystemToken')}
              value={form.systemAccessToken}
              onChange={(systemAccessToken) => setForm((current) => ({ ...current, systemAccessToken }))}
            />
            <TextFieldRow
              required
              label={t('providers.quickImportUserId')}
              value={form.userId}
              onChange={(userId) => setForm((current) => ({ ...current, userId }))}
            />
            <TextFieldRow
              required
              type="number"
              label={t('providers.quickImportRechargeMultiplier')}
              value={form.rechargeMultiplier}
              onChange={(rechargeMultiplier) => setForm((current) => ({ ...current, rechargeMultiplier }))}
            />
          </Stack>
        )}
      </Stack>
      <Divider />
      <Stack spacing={1.5}>
        <Typography variant="subtitle2">{t('providers.quickImportSyncSection')}</Typography>
        <ProviderQuickImportSyncConfigFields
          form={form.sync}
          onChange={(sync) => setForm((current) => ({ ...current, sync }))}
        />
      </Stack>
    </Stack>
  );
}

export function LocalKeyField({
  draft,
  selected,
  importable,
  localKeys,
  tokens,
  onUpdate,
}: {
  token: ProviderQuickImportTokenPreview;
  draft?: QuickImportTokenDraft;
  selected: boolean;
  importable: boolean;
  localKeys: ProviderQuickImportBindPreviewResponse['local_keys'];
  tokens: Record<string, QuickImportTokenDraft>;
  onUpdate: (patch: Partial<QuickImportTokenDraft>) => void;
}) {
  const { t } = useTranslate('admin');
  const selectedId = draft?.localKeyId ?? '';
  const selectedValue = selectedId || CREATE_LOCAL_KEY_VALUE;
  const usedIds = selectedLocalKeyIds(tokens);
  const options = localKeys.filter((key) => key.id === selectedId || !usedIds.has(key.id));
  const duplicate = Boolean(selectedId) && localKeyUseCount(tokens, selectedId) > 1;
  const createLabel = t('providers.quickImportBindCreateKey');

  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">
        {t('providers.quickImportBindLocalKey')}
      </Typography>
      <TextField
        select
        fullWidth
        size="small"
        value={selectedValue}
        disabled={!selected || !importable}
        error={selected && duplicate}
        SelectProps={{
          renderValue: (value) => localKeySelectLabel(String(value), localKeys, createLabel),
        }}
        onChange={(event) =>
          onUpdate({
            localKeyId: event.target.value === CREATE_LOCAL_KEY_VALUE ? '' : event.target.value,
          })
        }
      >
        <MenuItem value={CREATE_LOCAL_KEY_VALUE}>{createLabel}</MenuItem>
        {options.map((key) => (
          <MenuItem key={key.id} value={key.id}>
            {key.name}
          </MenuItem>
        ))}
      </TextField>
    </Stack>
  );
}

export function ConversionSummary({ summary }: { summary: BindSummary }) {
  const { t } = useTranslate('admin');

  return (
    <Alert severity="warning" variant="outlined">
      {t('providers.quickImportBindSummary', summary)}
    </Alert>
  );
}

export function bindSourceReady(form: QuickImportFormState) {
  return Boolean(
    form.sourceKind &&
      form.baseUrl.trim() &&
      (form.sourceKind === 'newapi'
        ? form.systemAccessToken.trim() && form.userId.trim()
        : form.sub2apiAuthTab === 'password'
          ? form.email.trim() && form.password.trim()
          : form.authToken.trim() && form.refreshToken.trim() && form.tokenExpiresAt.trim()) &&
      Number(form.rechargeMultiplier) > 0 &&
      validSyncConfig(form.sync)
  );
}

export function hasDuplicateLocalKey(
  selectedTokens: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>
) {
  const selectedIds = selectedTokens
    .map((token) => tokens[token.upstream_token_id]?.localKeyId)
    .filter(Boolean);

  return new Set(selectedIds).size !== selectedIds.length;
}

export function conversionSummary(
  preview: ProviderQuickImportBindPreviewResponse,
  selectedTokens: ProviderQuickImportTokenPreview[],
  tokens: Record<string, QuickImportTokenDraft>
) {
  const reusedIds = selectedTokens
    .map((token) => tokens[token.upstream_token_id]?.localKeyId)
    .filter(Boolean);

  return {
    reused: reusedIds.length,
    created: selectedTokens.length - reusedIds.length,
    deleted: Math.max(0, preview.local_keys.length - new Set(reusedIds).size),
  };
}

function selectedLocalKeyIds(tokens: Record<string, QuickImportTokenDraft>) {
  return new Set(
    Object.values(tokens)
      .filter((token) => token.selected)
      .map((token) => token.localKeyId)
      .filter(Boolean)
  );
}

function localKeyUseCount(tokens: Record<string, QuickImportTokenDraft>, id: string) {
  return Object.values(tokens).filter((token) => token.selected && token.localKeyId === id).length;
}

function localKeySelectLabel(
  value: string,
  localKeys: ProviderQuickImportBindPreviewResponse['local_keys'],
  createLabel: string
) {
  if (value === CREATE_LOCAL_KEY_VALUE) {
    return createLabel;
  }
  return localKeys.find((key) => key.id === value)?.name ?? value;
}
