'use client';

import type { Theme } from '@mui/material/styles';
import type { Dispatch, SetStateAction } from 'react';
import type { Provider } from 'src/types/provider';
import type { GlobalModelResponse } from 'src/types/model';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
} from 'src/types/provider-quick-import';

import { varAlpha } from 'minimal-shared/utils';
import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';
import {
  commitProviderQuickImportAppend,
  previewProviderQuickImportAppend,
} from 'src/actions/provider-quick-import';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { ProviderQuickImportPreviewStep } from './provider-quick-import-preview-step';
import { ProviderQuickImportMappingDialog } from './provider-quick-import-mapping-table';
import {
  defaultMappings,
  selectedTokenRows,
  globalModelHasCost,
  defaultTokenDrafts,
  appendCommitPayload,
  validCostMultiplier,
  type QuickImportTokenDraft,
  type QuickImportMappingsByToken,
} from './provider-quick-import-utils';

type Props = {
  open: boolean;
  provider: Provider | null;
  models: GlobalModelResponse[];
  onClose: () => void;
};

export function ProviderQuickImportAppendDialog({ open, provider, models, onClose }: Props) {
  const { t } = useTranslate('admin');
  const [preview, setPreview] = useState<ProviderQuickImportPreviewResponse | null>(null);
  const [tokens, setTokens] = useState<Record<string, QuickImportTokenDraft>>({});
  const [mappings, setMappings] = useState<QuickImportMappingsByToken>({});
  const [mappingTokenId, setMappingTokenId] = useState<string | null>(null);
  const [includeLinked, setIncludeLinked] = useState(false);
  const [loading, setLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const loadingKeyRef = useRef<string | null>(null);
  const selectedTokens = useMemo(() => selectedTokenRows(preview, tokens), [preview, tokens]);
  const mappingToken = useMemo(() => tokenById(preview, mappingTokenId), [mappingTokenId, preview]);
  const selectedGlobalModelIds = useMemo(
    () => mappedGlobalModelIds(selectedTokens, mappings),
    [mappings, selectedTokens]
  );
  const mappingMissing = selectedTokens.some((token) => tokenHasBlankMappings(token, mappings));
  const costMissing = selectedGlobalModelIds.some((id) => !globalModelHasCost(models, id));
  const disabled = commitDisabled(selectedTokens, selectedGlobalModelIds, tokens, mappings, costMissing || mappingMissing);

  const loadPreview = useCallback(async () => {
    if (!provider) return;
    const loadKey = `${provider.id}:${includeLinked}`;
    if (loadingKeyRef.current === loadKey) return;
    loadingKeyRef.current = loadKey;
    setLoading(true);
    try {
      const next = await previewProviderQuickImportAppend(provider.id, {
        include_linked_tokens: includeLinked,
      });
      setPreview(next);
      setTokens(defaultTokenDrafts(next));
      setMappings(defaultMappings(next));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadFailed'));
    } finally {
      loadingKeyRef.current = null;
      setLoading(false);
    }
  }, [includeLinked, provider, t]);

  useEffect(() => {
    if (open) void loadPreview();
  }, [loadPreview, open]);

  const close = () => {
    reset();
    onClose();
  };

  const submit = async () => {
    if (!provider || !preview || disabled || submitting) return;
    setSubmitting(true);
    try {
      await commitProviderQuickImportAppend(provider.id, appendCommitPayload(selectedTokens, tokens, mappings, models));
      toast.success(t('messages.providerQuickImportAppended'));
      close();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  };

  const reset = () => {
    setPreview(null);
    setTokens({});
    setMappings({});
    setMappingTokenId(null);
    setIncludeLinked(false);
    setLoading(false);
    setSubmitting(false);
    loadingKeyRef.current = null;
  };

  return (
    <Drawer anchor="right" open={open} onClose={close} slotProps={drawerSlotProps}>
      <DrawerHeader title={t('providers.quickImportAppendTitle', { name: provider?.name ?? '' })} onClose={close} />
      <Divider />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, py: 2.5, pb: 4 }}>
          <FormControlLabel
            control={<Switch checked={includeLinked} onChange={(event) => setIncludeLinked(event.target.checked)} />}
            label={t('providers.quickImportShowLinkedTokens')}
          />
          {loading ? <LoadingState /> : null}
          {!loading && preview ? (
            <ProviderQuickImportPreviewStep
              models={models}
              preview={preview}
              tokens={tokens}
              mappings={mappings}
              setTokens={setTokens}
              onMapModels={(token) => setMappingTokenId(token.upstream_token_id)}
            />
          ) : null}
          {!loading && preview?.tokens.length === 0 ? (
            <Alert severity="info">{t('providers.quickImportAppendEmpty')}</Alert>
          ) : null}
          {costMissing ? <Alert severity="error">{t('providers.quickImportCostMissing')}</Alert> : null}
        </Stack>
      </Scrollbar>
      <Divider />
      <Stack direction="row" spacing={1} justifyContent="flex-end" sx={{ px: 2.5, py: 2 }}>
        <Button variant="outlined" onClick={close}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} disabled={disabled} onClick={submit}>
          {t('providers.quickImportAppendSubmit')}
        </Button>
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

function LoadingState() {
  return (
    <Stack alignItems="center" justifyContent="center" sx={{ py: 6 }}>
      <CircularProgress size={24} />
    </Stack>
  );
}

function tokenById(preview: ProviderQuickImportPreviewResponse | null, id: string | null) {
  return preview?.tokens.find((token) => token.upstream_token_id === id);
}

function commitDisabled(
  selectedTokens: ProviderQuickImportTokenPreview[],
  selectedModelIds: string[],
  tokens: Record<string, QuickImportTokenDraft>,
  mappings: QuickImportMappingsByToken,
  mappingInvalid: boolean
) {
  return (
    selectedTokens.length === 0 ||
    selectedModelIds.length === 0 ||
    selectedTokens.some((token) => !(tokens[token.upstream_token_id]?.name ?? token.name).trim()) ||
    selectedTokens.some((token) => token.models.every((model) => !(model.upstream_model_id in (mappings[token.upstream_token_id] ?? {})))) ||
    selectedTokens.some((token) => tokens[token.upstream_token_id]?.endpointFormats.length === 0) ||
    selectedTokens.some((token) => !validCostMultiplier(tokens[token.upstream_token_id]?.costMultiplier)) ||
    mappingInvalid
  );
}

function tokenHasBlankMappings(token: ProviderQuickImportTokenPreview, mappingsByToken: QuickImportMappingsByToken) {
  const mappings = mappingsByToken[token.upstream_token_id] ?? {};
  return Object.keys(mappings).some((id) => token.models.some((model) => model.upstream_model_id === id) && !mappings[id]);
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
