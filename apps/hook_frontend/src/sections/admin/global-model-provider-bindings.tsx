'use client';

import type { CurrencyDisplay } from 'src/utils/currency-format';
import type { ModelCatalogProviderDetail } from 'src/types/model';

import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { formatPrice } from '../models/model-catalog-utils';

type Props = {
  providers: ModelCatalogProviderDetail[];
  loading: boolean;
  errorMessage?: string;
  currencyDisplay?: CurrencyDisplay;
};

export function GlobalModelProviderBindings({
  providers,
  loading,
  errorMessage,
  currencyDisplay,
}: Props) {
  const { t } = useTranslate('admin');
  const state = providerState(loading, errorMessage, providers.length, t);

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('providers.modelBindings')}</Typography>
      {state}
      {!loading && !errorMessage ? (
        <Stack spacing={1.25}>
          {providers.map((provider) => (
            <ProviderCard
              key={provider.model_id ?? `${provider.provider_id}-${provider.target_model}`}
              provider={provider}
              currencyDisplay={currencyDisplay}
            />
          ))}
        </Stack>
      ) : null}
    </Stack>
  );
}

function providerState(
  loading: boolean,
  errorMessage: string | undefined,
  length: number,
  t: (key: string) => string
) {
  if (loading) return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  if (errorMessage) return <Alert severity="error">{errorMessage}</Alert>;
  if (length === 0) return <Typography color="text.secondary">{t('common.noData')}</Typography>;
  return null;
}

function ProviderCard({
  provider,
  currencyDisplay,
}: {
  provider: ModelCatalogProviderDetail;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1} sx={providerCardSx}>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
        <Typography variant="subtitle2" noWrap sx={{ flexGrow: 1 }}>
          {provider.provider_name || provider.provider_id}
        </Typography>
        <Label color={provider.is_active ? 'success' : 'default'} variant="soft">
          {provider.is_active ? t('common.active') : t('common.disabled')}
        </Label>
      </Stack>
      <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
        {provider.target_model}
      </Typography>
      <ProviderPrice provider={provider} currencyDisplay={currencyDisplay} />
    </Stack>
  );
}

function ProviderPrice({
  provider,
  currencyDisplay,
}: {
  provider: ModelCatalogProviderDetail;
  currencyDisplay?: CurrencyDisplay;
}) {
  const { t } = useTranslate('admin');
  const rows = [
    [
      t('providers.inputOutputPrice'),
      `${formatPrice(provider.input_price_per_1m, currencyDisplay)} / ${formatPrice(provider.output_price_per_1m, currencyDisplay)}`,
    ],
    [
      t('providers.cachePrice'),
      `${formatPrice(provider.cache_creation_price_per_1m, currencyDisplay)} / ${formatPrice(provider.cache_read_price_per_1m, currencyDisplay)}`,
    ],
  ];

  if (provider.price_per_request && provider.price_per_request > 0) {
    rows.push([
      t('providers.pricePerRequest'),
      `${formatPrice(provider.price_per_request, currencyDisplay)}/${t('providers.perRequest')}`,
    ]);
  }

  return (
    <Stack spacing={0.5}>
      {rows.map(([label, value]) => <PriceLine key={label} label={label} value={value} />)}
    </Stack>
  );
}

function PriceLine({ label, value }: { label: string; value: string }) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={1.5}>
      <Typography variant="caption" color="text.secondary">{label}</Typography>
      <Typography variant="body2" sx={{ fontFamily: 'monospace', textAlign: 'right' }}>{value}</Typography>
    </Stack>
  );
}

const providerCardSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: { palette: { divider: string } }) => `1px solid ${theme.palette.divider}`,
};
