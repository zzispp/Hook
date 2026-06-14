'use client';

import type { Theme } from '@mui/material/styles';
import type { Provider } from 'src/types/provider';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderQuickImportBindPreviewResponse } from 'src/types/provider-quick-import';

import { useMemo, useState } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Step from '@mui/material/Step';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Stepper from '@mui/material/Stepper';
import Divider from '@mui/material/Divider';
import StepLabel from '@mui/material/StepLabel';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import {
  commitProviderQuickImportBind,
  previewProviderQuickImportBind,
} from 'src/actions/provider-quick-import';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { ConfirmDialog } from 'src/components/custom-dialog';

import { ProviderQuickImportPreviewStep } from './provider-quick-import-preview-step';
import { ProviderQuickImportMappingDialog } from './provider-quick-import-mapping-table';
import {
  LocalKeyField,
  BindSourceStep,
  bindSourceReady,
  ConversionSummary,
  conversionSummary,
  hasDuplicateLocalKey,
} from './provider-quick-import-bind-parts';
import {
  defaultMappings,
  validSyncConfig,
  bindCommitPayload,
  bindSourcePayload,
  selectedTokenRows,
  defaultTokenDrafts,
  globalModelHasCost,
  validCostMultiplier,
  DEFAULT_QUICK_IMPORT_FORM,
  type QuickImportTokenDraft,
  selectedMappedUpstreamModels,
} from './provider-quick-import-utils';

type Props = {
  open: boolean;
  provider: Provider | null;
  models: GlobalModelResponse[];
  onClose: () => void;
  onBound: () => void;
};

export function ProviderQuickImportBindDialog({ open, provider, models, onClose, onBound }: Props) {
  const { t } = useTranslate('admin');
  const [step, setStep] = useState(0);
  const [form, setForm] = useState(DEFAULT_QUICK_IMPORT_FORM);
  const [preview, setPreview] = useState<ProviderQuickImportBindPreviewResponse | null>(null);
  const [tokens, setTokens] = useState<Record<string, QuickImportTokenDraft>>({});
  const [mappings, setMappings] = useState<Record<string, string>>({});
  const [mappingTokenId, setMappingTokenId] = useState<string | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const selectedTokens = useMemo(() => selectedTokenRows(preview?.preview ?? null, tokens), [preview, tokens]);
  const selectedModelIds = useMemo(() => selectedMappedUpstreamModels(selectedTokens, mappings), [mappings, selectedTokens]);
  const mappingToken = useMemo(
    () => preview?.preview.tokens.find((token) => token.upstream_token_id === mappingTokenId),
    [mappingTokenId, preview]
  );
  const mappingMissing = selectedModelIds.some((id) => !mappings[id]);
  const costMissing = selectedModelIds.some((id) => !globalModelHasCost(models, mappings[id]));
  const duplicateLocalKey = hasDuplicateLocalKey(selectedTokens, tokens);
  const disabled =
    selectedTokens.length === 0 ||
    selectedModelIds.length === 0 ||
    selectedTokens.some((token) => !(tokens[token.upstream_token_id]?.name ?? token.name).trim()) ||
    selectedTokens.some((token) => token.models.every((model) => !(model.upstream_model_id in mappings))) ||
    selectedTokens.some((token) => tokens[token.upstream_token_id]?.endpointFormats.length === 0) ||
    selectedTokens.some((token) => !validCostMultiplier(tokens[token.upstream_token_id]?.costMultiplier)) ||
    !validSyncConfig(form.sync) ||
    duplicateLocalKey ||
    mappingMissing ||
    costMissing;
  const summary = preview ? conversionSummary(preview, selectedTokens, tokens) : null;

  const close = () => {
    reset();
    onClose();
  };

  const loadPreview = async () => {
    if (!provider || submitting) return;
    setSubmitting(true);
    try {
      const next = await previewProviderQuickImportBind(provider.id, bindSourcePayload(form));
      setPreview(next);
      setTokens(defaultTokenDrafts(next.preview));
      setMappings(defaultMappings(next.preview));
      setStep(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const submit = async () => {
    if (!provider || !preview || disabled || submitting) return;
    setSubmitting(true);
    try {
      await commitProviderQuickImportBind(provider.id, bindCommitPayload(form, selectedTokens, tokens, mappings, models));
      toast.success(t('messages.providerQuickImportBound'));
      onBound();
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
      setConfirmOpen(false);
    }
  };

  const reset = () => {
    setStep(0);
    setForm(DEFAULT_QUICK_IMPORT_FORM);
    setPreview(null);
    setTokens({});
    setMappings({});
    setMappingTokenId(null);
    setConfirmOpen(false);
    setSubmitting(false);
  };

  return (
    <Drawer anchor="right" open={open} onClose={close} slotProps={drawerSlotProps}>
      <DrawerHeader title={t('providers.quickImportBindTitle', { name: provider?.name ?? '' })} onClose={close} />
      <Divider />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, py: 2.5, pb: 4 }}>
          <Stepper activeStep={step}>
            <Step><StepLabel>{t('providers.quickImportSource')}</StepLabel></Step>
            <Step><StepLabel>{t('providers.quickImportBindPreview')}</StepLabel></Step>
          </Stepper>
          {step === 0 ? <BindSourceStep form={form} setForm={setForm} /> : null}
          {step === 1 && preview ? (
            <ProviderQuickImportPreviewStep
              models={models}
              preview={preview.preview}
              tokens={tokens}
              mappings={mappings}
              setTokens={setTokens}
              setMappings={setMappings}
              onMapModels={(token) => setMappingTokenId(token.upstream_token_id)}
              renderExtraFields={(input) => (
                <LocalKeyField
                  {...input}
                  localKeys={preview.local_keys}
                  tokens={tokens}
                />
              )}
            />
          ) : null}
          {duplicateLocalKey ? <Alert severity="error">{t('providers.quickImportBindDuplicateKey')}</Alert> : null}
          {costMissing ? <Alert severity="error">{t('providers.quickImportCostMissing')}</Alert> : null}
          {summary ? <ConversionSummary summary={summary} /> : null}
        </Stack>
      </Scrollbar>
      <Divider />
      <Stack direction="row" spacing={1} justifyContent="flex-end" sx={{ px: 2.5, py: 2 }}>
        {step === 1 ? <Button onClick={() => setStep(0)}>{t('common.back')}</Button> : null}
        <Button variant="outlined" onClick={close}>{t('common.cancel')}</Button>
        {step === 0 ? (
          <Button variant="contained" loading={submitting} disabled={!bindSourceReady(form)} onClick={loadPreview}>
            {t('common.next')}
          </Button>
        ) : (
          <Button variant="contained" color="error" disabled={disabled} onClick={() => setConfirmOpen(true)}>
            {t('providers.quickImportBindSubmit')}
          </Button>
        )}
      </Stack>
      {preview ? (
        <ProviderQuickImportMappingDialog
          open={!!mappingToken}
          preview={preview.preview}
          token={mappingToken}
          models={models}
          mappings={mappings}
          setMappings={setMappings}
          onClose={() => setMappingTokenId(null)}
        />
      ) : null}
      <ConfirmDialog
        open={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        title={t('providers.quickImportBindConfirmTitle')}
        content={summary ? t('providers.quickImportBindConfirmContent', summary) : ''}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" loading={submitting} onClick={submit}>
            {t('providers.quickImportBindSubmit')}
          </Button>
        }
      />
    </Drawer>
  );
}

function DrawerHeader({ title, onClose }: { title: string; onClose: () => void }) {
  return (
    <Box sx={headerSx}>
      <Typography variant="h6" noWrap sx={{ flexGrow: 1, minWidth: 0 }}>
        {title}
      </Typography>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Box>
  );
}

const drawerSlotProps = {
  backdrop: { invisible: true },
  paper: {
    sx: [
      (theme: Theme) => ({
        ...theme.mixins.paperStyles(theme, {
          color: varAlpha(theme.vars.palette.background.defaultChannel, 0.95),
        }),
        width: { xs: 1, sm: 820 },
      }),
    ],
  },
};

const headerSx = {
  py: 2,
  pr: 1,
  pl: 2.5,
  gap: 1,
  display: 'flex',
  alignItems: 'center',
};
