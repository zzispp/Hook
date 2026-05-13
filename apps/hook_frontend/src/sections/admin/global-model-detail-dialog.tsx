'use client';

import type { ComponentProps } from 'react';
import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
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
import { useGlobalModelProviders } from 'src/actions/models';
import { useSystemSettings, useUsdCnyExchangeRate } from 'src/actions/system-settings';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { ModelCopyButton } from '../models/model-copy-button';
import { formatUsageCount } from '../models/model-catalog-utils';
import { ModelPricingSection } from '../models/model-pricing-section';
import { GlobalModelProviderBindings } from './global-model-provider-bindings';
import { formFromModel, capabilitiesFromForm } from './model-management-utils';
import { GlobalModelBillingGroupPricing } from './global-model-billing-group-pricing';

type Props = {
  model: GlobalModelResponse | null;
  groups: BillingGroup[];
  groupsLoading: boolean;
  groupsErrorMessage?: string;
  open: boolean;
  onClose: () => void;
  onEdit: (model: GlobalModelResponse) => void;
};

type DetailTab = 'basic' | 'groups' | 'providers';
type IconifyIconName = ComponentProps<typeof Iconify>['icon'];

export function GlobalModelDetailDialog(props: Props) {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<DetailTab>('basic');
  const providers = useGlobalModelProviders(props.open ? props.model?.id : null);
  const currency = useDetailCurrency(props.open);

  if (!props.model) return null;

  return (
    <Dialog open={props.open} fullWidth maxWidth="md" onClose={props.onClose}>
      <DetailHeader model={props.model} onClose={props.onClose} onEdit={props.onEdit} />
      <DialogContent dividers sx={{ p: 0 }}>
        <Tabs value={tab} onChange={(_, value: DetailTab) => setTab(value)} sx={tabsSx}>
          <Tab value="basic" label={t('models.basic')} />
          <Tab value="groups" label={t('models.groupPricing')} />
          <Tab value="providers" label={t('models.providers')} />
        </Tabs>
        <Stack spacing={2.5} sx={{ p: 2.5 }}>
          <CurrencyState currency={currency} />
          {tab === 'basic' ? <BasicTab model={props.model} currencyDisplay={currency.display} /> : null}
          {tab === 'groups' ? (
            <GlobalModelBillingGroupPricing
              model={props.model}
              groups={props.groups}
              loading={props.groupsLoading}
              errorMessage={props.groupsErrorMessage}
              currencyDisplay={currency.display}
            />
          ) : null}
          {tab === 'providers' ? (
            <GlobalModelProviderBindings
              providers={providers.items}
              loading={providers.isLoading}
              errorMessage={providers.error?.message}
              currencyDisplay={currency.display}
            />
          ) : null}
        </Stack>
      </DialogContent>
    </Dialog>
  );
}

function DetailHeader({
  model,
  onClose,
  onEdit,
}: {
  model: GlobalModelResponse;
  onClose: () => void;
  onEdit: (model: GlobalModelResponse) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <DialogTitle sx={titleSx}>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
          <Typography variant="h6" noWrap>{model.display_name}</Typography>
          <Label color={model.is_active ? 'success' : 'default'} variant="soft">
            {model.is_active ? t('models.available') : t('models.unavailable')}
          </Label>
        </Stack>
        <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
          <Typography variant="caption" color="text.secondary" sx={modelIdSx}>{model.name}</Typography>
          <ModelCopyButton value={model.name} />
        </Stack>
      </Box>
      <HeaderButton title={t('common.edit')} icon="solar:pen-bold" onClick={() => onEdit(model)} />
      <HeaderButton title={t('common.close')} icon="mingcute:close-line" onClick={onClose} />
    </DialogTitle>
  );
}

function HeaderButton({
  title,
  icon,
  onClick,
}: {
  title: string;
  icon: IconifyIconName;
  onClick: () => void;
}) {
  return (
    <Tooltip title={title}>
      <IconButton onClick={onClick}>
        <Iconify icon={icon} />
      </IconButton>
    </Tooltip>
  );
}

function BasicTab({
  model,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
}) {
  const capabilities = capabilitiesFromForm(formFromModel(model));

  return (
    <Stack spacing={2.5}>
      <SummaryGrid model={model} capabilities={capabilities} currencyDisplay={currencyDisplay} />
      <Divider />
      <ModelPricingSection model={model} currencyDisplay={currencyDisplay} />
    </Stack>
  );
}

function SummaryGrid({
  model,
  capabilities,
  currencyDisplay,
}: {
  model: GlobalModelResponse;
  capabilities: string[];
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const items = [
    [t('fields.createdAt'), fDateTime(model.created_at)],
    [t('systemSettings.fields.currency'), currencyDisplay?.currency ?? t('common.loading')],
    [t('models.providers'), `${model.active_provider_count ?? 0} / ${model.provider_count ?? 0}`],
    [t('models.usageCount'), formatUsageCount(model.usage_count)],
  ];

  return (
    <Box sx={summaryGridSx}>
      {items.map(([label, value]) => <SummaryItem key={label} label={label} value={value} />)}
      <CapabilitySummary capabilities={capabilities} />
    </Box>
  );
}

function SummaryItem({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.5} sx={summaryItemSx}>
      <Typography variant="caption" color="text.secondary">{label}</Typography>
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>{value || '-'}</Typography>
    </Stack>
  );
}

function CapabilitySummary({ capabilities }: { capabilities: string[] }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={0.75} sx={summaryItemSx}>
      <Typography variant="caption" color="text.secondary">{t('models.capabilities')}</Typography>
      <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75 }}>
        {capabilities.length === 0 ? <Typography variant="body2">-</Typography> : null}
        {capabilities.map((capability) => (
          <Label key={capability} variant="soft">{t(`models.capability.${capability}`)}</Label>
        ))}
      </Stack>
    </Stack>
  );
}

function useDetailCurrency(enabled: boolean) {
  const { t } = useTranslate('admin');
  const settings = useSystemSettings(enabled);
  const needsExchangeRate = enabled && settings.data?.currency === 'CNY';
  const exchangeRate = useUsdCnyExchangeRate(needsExchangeRate);

  return {
    display: settings.data
      ? ({
          currency: settings.data.currency,
          usdCnyRate: exchangeRate.data,
          unavailableLabel: t('requestRecords.exchangeRateUnavailable'),
        } satisfies CurrencyDisplay)
      : undefined,
    loading: settings.isLoading || (needsExchangeRate && exchangeRate.isLoading),
    error: settings.error ?? exchangeRate.error,
  };
}

function CurrencyState({
  currency,
}: {
  currency: ReturnType<typeof useDetailCurrency>;
}) {
  const { t } = useTranslate('admin');
  if (currency.error) return <Alert severity="error">{currency.error.message}</Alert>;
  if (currency.loading) return <Alert severity="info">{t('common.loading')}</Alert>;
  return null;
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

const modelIdSx = {
  minWidth: 0,
  fontFamily: 'monospace',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
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
