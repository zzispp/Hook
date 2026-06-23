'use client';

import type { Theme } from '@mui/material/styles';
import type { Dispatch, SetStateAction } from 'react';
import type { GlobalModelResponse } from 'src/types/model';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
} from 'src/types/provider-quick-import';

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
import { commitProviderQuickImport, previewProviderQuickImport } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { ProviderQuickImportSourceStep } from './provider-quick-import-source-step';
import { ProviderQuickImportPreviewStep } from './provider-quick-import-preview-step';
import { ProviderQuickImportMappingDialog } from './provider-quick-import-mapping-table';
import {
  sourceReady,
  commitPayload,
  previewPayload,
  defaultMappings,
  validSyncConfig,
  selectedTokenRows,
  globalModelHasCost,
  defaultTokenDrafts,
  validCostMultiplier,
  DEFAULT_QUICK_IMPORT_FORM,
  type QuickImportTokenDraft,
  selectedMappedUpstreamModels,
  type QuickImportMappingsByToken,
} from './provider-quick-import-utils';

type Props = {
  open: boolean;
  models: GlobalModelResponse[];
  onClose: () => void;
  onImported: () => void;
};

export function ProviderQuickImportDialog({ open, models, onClose, onImported }: Props) {
  const { t } = useTranslate('admin');
  const [step, setStep] = useState(0);
  const [form, setForm] = useState(DEFAULT_QUICK_IMPORT_FORM);
  const [preview, setPreview] = useState<ProviderQuickImportPreviewResponse | null>(null);
  const [tokens, setTokens] = useState<Record<string, QuickImportTokenDraft>>({});
  const [mappings, setMappings] = useState<QuickImportMappingsByToken>({});
  const [mappingTokenId, setMappingTokenId] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const selectedTokens = useMemo(() => selectedTokenRows(preview, tokens), [preview, tokens]);
  const mappingToken = useMemo(
    () => preview?.tokens.find((token) => token.upstream_token_id === mappingTokenId),
    [mappingTokenId, preview]
  );
  const selectedGlobalModelIds = useMemo(
    () => mappedGlobalModelIds(selectedTokens, mappings),
    [mappings, selectedTokens]
  );
  const mappingMissing = selectedTokens.some((token) => tokenHasBlankMappings(token, mappings));
  const costMissing = selectedGlobalModelIds.some((id) => !globalModelHasCost(models, id));
  const commitDisabled =
    selectedTokens.length === 0 ||
    selectedGlobalModelIds.length === 0 ||
    selectedTokens.some((token) => !(tokens[token.upstream_token_id]?.name ?? token.name).trim()) ||
    selectedTokens.some((token) => tokenMappedModelCount(token, mappings[token.upstream_token_id] ?? {}) === 0) ||
    selectedTokens.some((token) => tokens[token.upstream_token_id]?.endpointFormats.length === 0) ||
    selectedTokens.some((token) => !validCostMultiplier(tokens[token.upstream_token_id]?.costMultiplier)) ||
    !validSyncConfig(form.sync) ||
    mappingMissing ||
    costMissing;

  const close = () => {
    reset();
    onClose();
  };

  const previewImport = async () => {
    if (submitting) return;
    setSubmitting(true);
    try {
      const next = await previewProviderQuickImport(previewPayload(form));
      setPreview(next);
      setTokens(defaultTokenDrafts(next));
      setMappings(defaultMappings(next));
      setStep(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const commitImport = async () => {
    if (!preview || commitDisabled || submitting) return;
    setSubmitting(true);
    try {
      await commitProviderQuickImport(commitPayload(form, selectedTokens, tokens, mappings, models));
      toast.success(t('messages.providerQuickImportImported'));
      onImported();
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const reset = () => {
    setStep(0);
    setForm(DEFAULT_QUICK_IMPORT_FORM);
    setPreview(null);
    setTokens({});
    setMappings({});
    setMappingTokenId(null);
    setSubmitting(false);
  };

  return (
    <Drawer anchor="right" open={open} onClose={close} slotProps={drawerSlotProps}>
      <QuickImportDrawerHeader title={t('dialogs.quickImportProvider')} onClose={close} />
      <Divider />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, py: 2.5, pb: 4 }}>
          <Stepper activeStep={step}>
            <Step><StepLabel>{t('providers.quickImportSource')}</StepLabel></Step>
            <Step><StepLabel>{t('providers.quickImportPreview')}</StepLabel></Step>
          </Stepper>
          {step === 0 ? <ProviderQuickImportSourceStep form={form} setForm={setForm} /> : null}
          {step === 1 && preview ? (
            <ProviderQuickImportPreviewStep
              models={models}
              preview={preview}
              tokens={tokens}
              mappings={mappings}
              setTokens={setTokens}
              onMapModels={(token) => setMappingTokenId(token.upstream_token_id)}
            />
          ) : null}
          {step === 1 && costMissing ? <Alert severity="error">{t('providers.quickImportCostMissing')}</Alert> : null}
        </Stack>
      </Scrollbar>
      <Divider />
      <Stack direction="row" spacing={1} justifyContent="flex-end" sx={{ px: 2.5, py: 2 }}>
        {step === 1 ? <Button onClick={() => setStep(0)}>{t('common.back')}</Button> : null}
        <Button variant="outlined" onClick={close}>{t('common.cancel')}</Button>
        {step === 0 ? (
          <Button variant="contained" loading={submitting} disabled={!sourceReady(form)} onClick={previewImport}>
            {t('common.next')}
          </Button>
        ) : (
          <Button variant="contained" loading={submitting} disabled={commitDisabled} onClick={commitImport}>
            {t('providers.quickImportSubmit')}
          </Button>
        )}
      </Stack>
      {preview ? (
        <ProviderQuickImportMappingDialog
          open={!!mappingToken}
          preview={preview}
          token={mappingToken}
          models={models}
          mappings={mappingToken ? mappings[mappingToken.upstream_token_id] ?? {} : {}}
          setMappings={tokenMappingsSetter(mappingToken, setMappings)}
          onClose={() => setMappingTokenId(null)}
        />
      ) : null}
    </Drawer>
  );
}

function QuickImportDrawerHeader({ title, onClose }: { title: string; onClose: () => void }) {
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

function tokenMappedModelCount(token: ProviderQuickImportTokenPreview, mappings: Record<string, string>) {
  return token.models.filter((model) => model.upstream_model_id in mappings).length;
}

function tokenHasBlankMappings(token: ProviderQuickImportTokenPreview, mappingsByToken: QuickImportMappingsByToken) {
  return selectedMappedUpstreamModels([token], mappingsByToken[token.upstream_token_id] ?? {}).some(
    (id) => !(mappingsByToken[token.upstream_token_id] ?? {})[id]
  );
}

function mappedGlobalModelIds(tokens: ProviderQuickImportTokenPreview[], mappingsByToken: QuickImportMappingsByToken) {
  return [...new Set(tokens.flatMap((token) => Object.values(mappingsByToken[token.upstream_token_id] ?? {}).filter(Boolean)))];
}

function tokenMappingsSetter(
  token: ProviderQuickImportTokenPreview | undefined,
  setMappings: Dispatch<SetStateAction<QuickImportMappingsByToken>>
) {
  return (updater: SetStateAction<Record<string, string>>) => {
    if (!token) return;
    setMappings((current) => {
      const currentTokenMappings = current[token.upstream_token_id] ?? {};
      const nextTokenMappings =
        typeof updater === 'function' ? updater(currentTokenMappings) : updater;
      return {
        ...current,
        [token.upstream_token_id]: nextTokenMappings,
      };
    });
  };
}

const drawerSlotProps = {
  backdrop: { invisible: true },
  paper: {
    sx: [
      (theme: Theme) => ({
        ...theme.mixins.paperStyles(theme, {
          color: varAlpha(theme.vars.palette.background.defaultChannel, 0.95),
        }),
        width: { xs: 1, sm: 760 },
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
