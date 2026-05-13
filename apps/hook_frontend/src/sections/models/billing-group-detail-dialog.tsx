'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { ModelCopyButton } from './model-copy-button';
import { ModelPricingSection } from './model-pricing-section';
import { ModelEffectivePricingSection } from './model-effective-pricing-section';

type Props = {
  group: BillingGroup | null;
  models: GlobalModelResponse[];
  currencyDisplay?: CurrencyDisplay;
  currencyErrorMessage?: string;
  open: boolean;
  onClose: () => void;
};

export function BillingGroupDetailDialog({
  group,
  models,
  currencyDisplay,
  currencyErrorMessage,
  open,
  onClose,
}: Props) {
  if (!group) return null;

  const scopedModels = modelsForGroup(group, models);

  return (
    <Dialog open={open} fullWidth maxWidth="md" onClose={onClose}>
      <DialogHeader group={group} onClose={onClose} />
      <DialogContent dividers>
        <Stack spacing={2.5}>
          <SummaryGrid group={group} modelCount={scopedModels.length} />
          <Description group={group} />
          {currencyErrorMessage ? <Alert severity="error">{currencyErrorMessage}</Alert> : null}
          <ModelsPricingList
            group={group}
            models={scopedModels}
            currencyDisplay={currencyDisplay}
          />
        </Stack>
      </DialogContent>
    </Dialog>
  );
}

function DialogHeader({ group, onClose }: { group: BillingGroup; onClose: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <DialogTitle sx={titleSx}>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="h6" noWrap>
            {group.name}
          </Typography>
          {group.is_system ? <Label variant="soft">{t('common.system')}</Label> : null}
          <Label color="info" variant="soft">
            x{formatMultiplier(group.billing_multiplier)}
          </Label>
        </Stack>
        <Typography variant="caption" color="text.secondary" sx={codeSx}>
          {group.code}
        </Typography>
      </Box>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </DialogTitle>
  );
}

function SummaryGrid({
  group,
  modelCount,
}: {
  group: BillingGroup;
  modelCount: number;
}) {
  const { t } = useTranslate('admin');
  const modelScope = group.allowed_model_ids.length === 0
    ? t('billingGroups.allModels')
    : t('billingGroups.selectedModelCount', { count: modelCount });
  const items = [
    [t('fields.billingMultiplier'), `x${formatMultiplier(group.billing_multiplier)}`],
    [t('fields.allowedModels'), modelScope],
  ];

  return (
    <Box sx={summaryGridSx}>
      {items.map(([label, value]) => (
        <Stack key={label} spacing={0.5} sx={summaryItemSx}>
          <Typography variant="caption" color="text.secondary">
            {label}
          </Typography>
          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
            {value}
          </Typography>
        </Stack>
      ))}
    </Box>
  );
}

function Description({ group }: { group: BillingGroup }) {
  if (!group.description) return null;

  return (
    <Typography variant="body2" color="text.secondary">
      {group.description}
    </Typography>
  );
}

function ModelsPricingList({
  group,
  models,
  currencyDisplay,
}: {
  group: BillingGroup;
  models: GlobalModelResponse[];
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  if (models.length === 0) {
    return <Typography color="text.secondary">{t('common.noData')}</Typography>;
  }

  return (
    <Stack spacing={1.5}>
      {models.map((model) => (
        <ModelPricingCard
          key={model.id}
          group={group}
          model={model}
          currencyDisplay={currencyDisplay}
        />
      ))}
    </Stack>
  );
}

function ModelPricingCard({
  group,
  model,
  currencyDisplay,
}: {
  group: BillingGroup;
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2} sx={panelSx}>
      <Stack spacing={0.75}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap sx={{ flexGrow: 1 }}>
            {model.display_name}
          </Typography>
          <Label color={model.is_active ? 'success' : 'default'} variant="soft">
            {model.is_active ? t('models.available') : t('models.unavailable')}
          </Label>
        </Stack>
        <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
          <Typography variant="caption" color="text.secondary" sx={modelIdSx}>
            {model.name}
          </Typography>
          <ModelCopyButton value={model.name} />
        </Stack>
        {typeof model.config?.description === 'string' && model.config.description.trim() ? (
          <Typography variant="body2" color="text.secondary">
            {model.config.description}
          </Typography>
        ) : null}
      </Stack>

      <Divider />
      <ModelPricingSection model={model} currencyDisplay={currencyDisplay} />
      <Divider />
      <ModelEffectivePricingSection
        model={model}
        multiplier={group.billing_multiplier}
        currencyDisplay={currencyDisplay}
        title={t('models.priceGroupPricing')}
      />
    </Stack>
  );
}

function modelsForGroup(group: BillingGroup, models: GlobalModelResponse[]) {
  if (group.allowed_model_ids.length === 0) return models;

  const allowed = new Set(group.allowed_model_ids);
  return models.filter((model) => allowed.has(model.id));
}

function formatMultiplier(value: number) {
  return Number.isInteger(value)
    ? String(value)
    : value.toFixed(6).replace(/0+$/, '').replace(/\.$/, '');
}

const titleSx = {
  display: 'flex',
  alignItems: 'flex-start',
  gap: 1,
};

const codeSx = {
  fontFamily: 'monospace',
};

const summaryGridSx = {
  display: 'grid',
  gridTemplateColumns: { xs: '1fr', sm: 'repeat(2, minmax(0, 1fr))' },
  gap: 1.5,
};

const summaryItemSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const modelIdSx = {
  minWidth: 0,
  fontFamily: 'monospace',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};
