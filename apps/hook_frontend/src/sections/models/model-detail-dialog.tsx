'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { fDateTime } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';
import { useAvailableBillingGroups } from 'src/actions/groups';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { ModelCopyButton } from './model-copy-button';
import { ModelPricingSection } from './model-pricing-section';
import { GlobalModelBillingGroupPricing } from '../admin/global-model-billing-group-pricing';
import {
  hasCapability,
  formatUsageCount,
  MODEL_DETAIL_CAPABILITIES,
} from './model-catalog-utils';

type Props = {
  model: GlobalModelResponse | null;
  currencyDisplay?: CurrencyDisplay;
  open: boolean;
  onClose: () => void;
};

type DetailTab = 'basic' | 'groups';

export function ModelDetailDialog({
  model,
  currencyDisplay,
  open,
  onClose,
}: Props) {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<DetailTab>('basic');
  const groups = useAvailableBillingGroups(open);

  if (!model) return null;

  return (
    <Dialog open={open} fullWidth maxWidth="md" onClose={onClose}>
      <DialogHeader model={model} onClose={onClose} />
      <DialogContent dividers sx={{ p: 0 }}>
        <Tabs value={tab} onChange={(_, value: DetailTab) => setTab(value)} sx={tabsSx}>
          <Tab value="basic" label={t('models.basic')} />
          <Tab value="groups" label={t('models.priceGroupPricing')} />
        </Tabs>
        <Stack spacing={2.5} sx={{ p: 2.5 }}>
          {tab === 'basic' ? <BasicTab model={model} currencyDisplay={currencyDisplay} /> : null}
          {tab === 'groups' ? (
            <GlobalModelBillingGroupPricing
              model={model}
              groups={groups.items}
              loading={groups.isLoading}
              errorMessage={groups.error?.message}
              currencyDisplay={currencyDisplay}
              title={t('models.priceGroupPricing')}
            />
          ) : null}
        </Stack>
      </DialogContent>
    </Dialog>
  );
}

function DialogHeader({
  model,
  onClose,
}: {
  model: GlobalModelResponse;
  onClose: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <DialogTitle sx={titleSx}>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="h6" noWrap>
            {model.display_name}
          </Typography>
          <Label color={model.is_active ? 'success' : 'default'} variant="soft">
            {model.is_active ? t('models.available') : t('models.unavailable')}
          </Label>
        </Stack>
        <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
          <Typography variant="caption" color="text.secondary" sx={modelNameSx}>
            {model.name}
          </Typography>
          <ModelCopyButton value={model.name} />
        </Stack>
      </Box>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </DialogTitle>
  );
}

function BasicTab({
  model,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  return (
    <Stack spacing={2.5}>
      <SummaryGrid model={model} currencyDisplay={currencyDisplay} />
      <Description model={model} />
      <CapabilitySection model={model} />
      <Divider />
      <ModelPricingSection model={model} currencyDisplay={currencyDisplay} />
    </Stack>
  );
}

function SummaryGrid({
  model,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const items = [
    [t('fields.createdAt'), fDateTime(model.created_at)],
    [t('systemSettings.fields.currency'), currencyDisplay?.currency ?? t('common.loading')],
    [t('models.usageCount'), formatUsageCount(model.usage_count)],
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

function Description({ model }: { model: GlobalModelResponse }) {
  const value = model.config?.description;
  if (typeof value !== 'string' || !value.trim()) return null;

  return (
    <Typography variant="body2" color="text.secondary">
      {value}
    </Typography>
  );
}

function CapabilitySection({ model }: { model: GlobalModelResponse }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('models.capabilities')}</Typography>
      <Stack spacing={1.25}>
        {MODEL_DETAIL_CAPABILITIES.map((item) => (
          <Stack
            key={item.key}
            direction="row"
            alignItems="center"
            spacing={1.5}
            sx={capabilityPanelSx}
          >
            <Iconify
              width={22}
              icon={
                hasCapability(model, item.key)
                  ? 'solar:check-circle-bold'
                  : 'solar:forbidden-circle-bold'
              }
            />
            <Box sx={{ flexGrow: 1, minWidth: 0 }}>
              <Typography variant="subtitle2" noWrap>
                {item.title}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {t(item.descriptionKey)}
              </Typography>
            </Box>
            <Label color={hasCapability(model, item.key) ? 'success' : 'default'} variant="soft">
              {hasCapability(model, item.key) ? t('models.supported') : t('models.unsupported')}
            </Label>
          </Stack>
        ))}
      </Stack>
    </Stack>
  );
}

const titleSx = {
  display: 'flex',
  alignItems: 'flex-start',
  gap: 1,
};

const tabsSx = {
  px: 2.5,
  borderBottom: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const summaryGridSx = {
  display: 'grid',
  gridTemplateColumns: { xs: '1fr', sm: 'repeat(3, minmax(0, 1fr))' },
  gap: 1.5,
};

const summaryItemSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const modelNameSx = {
  minWidth: 0,
  fontFamily: 'monospace',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};

const capabilityPanelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};
