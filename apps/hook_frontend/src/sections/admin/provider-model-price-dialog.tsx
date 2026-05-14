'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse, TieredPricingConfig } from 'src/types/model';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogActions from '@mui/material/DialogActions';

import { accountingCurrencyLabel } from 'src/utils/money-boundary';

import { updateGlobalModel } from 'src/actions/models';
import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { TieredPricingEditor } from './tiered-pricing-editor';
import { finalPricingConfig, normalizePricingConfig } from './tiered-pricing-utils';

type Props = {
  model: GlobalModelResponse | null;
  onClose: () => void;
};

export function GlobalModelPriceDialog({ model, onClose }: Props) {
  const { t } = useTranslate('admin');
  const form = usePriceForm(model);

  if (!model) return null;

  const submit = async () => {
    form.setSaving(true);
    try {
      await updateGlobalModel(model.id, pricePayload(form.pricing, form.pricePerRequest));
      toast.success(t('messages.modelUpdated'));
      onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      form.setSaving(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="md" open onClose={onClose} PaperProps={{ sx: dialogPaperSx }}>
      <DialogHeader onClose={onClose} />
      <Box sx={contentSx}>
        <ModelSummary model={model} />
        <Stack spacing={2}>
          <Typography variant="subtitle2" sx={sectionTitleSx}>
            {t('models.pricing')}
          </Typography>
          <TieredPricingEditor pricing={form.pricing} onChange={form.setPricing} />
          <RequestPriceField value={form.pricePerRequest} onChange={form.setPricePerRequest} />
        </Stack>
      </Box>
      <DialogActions sx={footerSx}>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={form.saving} onClick={submit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DialogHeader({ onClose }: { onClose: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center" spacing={1.5} sx={headerSx}>
      <Box sx={headerIconSx}>
        <Iconify icon="solar:pen-bold" width={22} />
      </Box>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6" noWrap>
          {t('dialogs.editProviderModel')}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {t('providers.modelPricingHint')}
        </Typography>
      </Box>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

function ModelSummary({ model }: { model: GlobalModelResponse }) {
  return (
    <Box sx={summarySx}>
      <Typography variant="h6" noWrap>
        {model.display_name}
      </Typography>
      <Typography variant="body2" noWrap sx={{ fontFamily: 'monospace', color: 'text.secondary' }}>
        {model.name}
      </Typography>
    </Box>
  );
}

function RequestPriceField({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1.5} alignItems={{ xs: 'stretch', sm: 'center' }} sx={requestPriceSx}>
      <Typography variant="caption" sx={{ fontWeight: 700, textTransform: 'uppercase', color: 'text.secondary' }}>
        {accountingCurrencyLabel(t('fields.pricePerRequest'))}
      </Typography>
      <TextField
        size="small"
        type="number"
        value={value}
        placeholder={t('providers.blankClearsPrice')}
        onChange={(event) => onChange(event.target.value)}
        inputProps={{ min: 0, step: 0.001 }}
        sx={{ width: { xs: 1, sm: 160 } }}
      />
      <Typography variant="caption" color="text.secondary">
        {t('providers.requestPriceHint')}
      </Typography>
    </Stack>
  );
}

function usePriceForm(model: GlobalModelResponse | null) {
  const [pricing, setPricing] = useState<TieredPricingConfig>(() => normalizePricingConfig(model?.default_tiered_pricing));
  const [pricePerRequest, setPricePerRequest] = useState('');
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setPricing(normalizePricingConfig(model?.default_tiered_pricing));
    setPricePerRequest(optionalNumberText(model?.default_price_per_request));
    setSaving(false);
  }, [model]);

  return { pricing, setPricing, pricePerRequest, setPricePerRequest, saving, setSaving };
}

function pricePayload(pricing: TieredPricingConfig, pricePerRequest: string) {
  return {
    default_tiered_pricing: finalPricingConfig(pricing),
    default_price_per_request: optionalNumberOrNull(pricePerRequest),
  };
}

function optionalNumberOrNull(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed) || parsed < 0) {
    throw new Error('Invalid numeric field');
  }
  return parsed;
}

function optionalNumberText(value: number | null | undefined) {
  return value === null || value === undefined ? '' : String(value);
}

const dialogPaperSx = { borderRadius: 1.5, overflow: 'hidden' };
const headerSx = { px: 3, py: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const headerIconSx = { width: 36, height: 36, borderRadius: 1, display: 'grid', placeItems: 'center', color: 'primary.main', bgcolor: 'primary.lighter' };
const contentSx = { px: 3, py: 2, display: 'grid', gap: 2 };
const summarySx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1, p: 2, bgcolor: 'background.neutral' };
const sectionTitleSx = { pb: 1, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const requestPriceSx = { pt: 1.5, borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const footerSx = { px: 3, py: 2, borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, bgcolor: 'background.neutral' };
