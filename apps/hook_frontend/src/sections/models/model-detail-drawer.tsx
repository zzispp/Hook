'use client';

import type { Theme } from '@mui/material/styles';
import type { PricingTier, GlobalModelResponse } from 'src/types/model';

import { varAlpha } from 'minimal-shared/utils';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import {
  tierCount,
  requestPrice,
  hasCapability,
  firstTierPrice,
  oneHourCachePrice,
  firstOneHourCachePrice,
  MODEL_DETAIL_CAPABILITIES,
} from './model-catalog-utils';

// ----------------------------------------------------------------------

type Props = {
  model: GlobalModelResponse | null;
  open: boolean;
  onClose: () => void;
  onExited: () => void;
};

export function ModelDetailDrawer({ model, open, onClose, onExited }: Props) {
  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      slotProps={drawerSlotProps(onExited)}
    >
      {model ? (
        <>
          <DrawerHeader onClose={onClose} />
          <Scrollbar>
            <Stack spacing={3} sx={contentSx}>
              <ModelSummary model={model} />
              <CapabilitySection model={model} />
              <PricingSection model={model} />
            </Stack>
          </Scrollbar>
        </>
      ) : null}
    </Drawer>
  );
}

function DrawerHeader({ onClose }: { onClose: () => void }) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={headerSx}>
      <Typography variant="h6" sx={{ flexGrow: 1 }}>
        {t('models.detailTitle')}
      </Typography>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function ModelSummary({ model }: { model: GlobalModelResponse }) {
  const { t } = useTranslate('admin');
  const { copy } = useCopyToClipboard();

  return (
    <Stack spacing={1.25}>
      <Typography variant="h5" sx={{ minWidth: 0 }}>
        {model.display_name || model.name}
      </Typography>
      <Stack direction="row" alignItems="center" spacing={1} sx={{ minWidth: 0 }}>
        <Label color={model.is_active ? 'success' : 'default'} variant="soft">
          {model.is_active ? t('models.available') : t('models.unavailable')}
        </Label>
        <Typography variant="body2" color="text.secondary" sx={modelNameSx}>
          {model.name}
        </Typography>
        <IconButton size="small" onClick={() => copyModelName(copy, model.name, t('models.modelIdCopied'))}>
          <Iconify width={16} icon="solar:copy-bold" />
        </IconButton>
      </Stack>
      <Description model={model} />
    </Stack>
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
      <Grid container spacing={1.5}>
        {MODEL_DETAIL_CAPABILITIES.map((item) => (
          <Grid key={item.key} size={{ xs: 12 }}>
            <CapabilityItem
              title={item.title}
              description={t(item.descriptionKey)}
              supported={hasCapability(model, item.key)}
            />
          </Grid>
        ))}
      </Grid>
    </Stack>
  );
}

function CapabilityItem({
  title,
  description,
  supported,
}: {
  title: string;
  description: string;
  supported: boolean;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center" spacing={1.5} sx={capabilityPanelSx}>
      <Iconify width={22} icon={supported ? 'solar:check-circle-bold' : 'solar:forbidden-circle-bold'} />
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="subtitle2" noWrap>
          {title}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          {description}
        </Typography>
      </Box>
      <Label color={supported ? 'success' : 'default'} variant="soft">
        {supported ? t('models.supported') : t('models.unsupported')}
      </Label>
    </Stack>
  );
}

function PricingSection({ model }: { model: GlobalModelResponse }) {
  const { t } = useTranslate('admin');
  const pricing = model.default_tiered_pricing;

  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('models.pricingInfo')}</Typography>
      {tierCount(pricing) <= 1 ? <SingleTierPricing model={model} /> : <TieredPricing tiers={pricing.tiers} />}
      <RequestPrice value={model.default_price_per_request} />
    </Stack>
  );
}

function SingleTierPricing({ model }: { model: GlobalModelResponse }) {
  const { t } = useTranslate('admin');
  const pricing = model.default_tiered_pricing;
  const items = [
    [t('fields.inputPrice'), firstTierPrice(pricing, 'input_price_per_1m')],
    [t('fields.outputPrice'), firstTierPrice(pricing, 'output_price_per_1m')],
    [t('fields.cacheCreationPrice'), firstTierPrice(pricing, 'cache_creation_price_per_1m')],
    [t('fields.cacheReadPrice'), firstTierPrice(pricing, 'cache_read_price_per_1m')],
  ];

  return (
    <Grid container spacing={1.5}>
      {items.map(([label, value]) => (
        <Grid key={label} size={{ xs: 12 }}>
          <PriceBox label={label} value={value} />
        </Grid>
      ))}
      <Grid size={{ xs: 12 }}>
        <InlinePrice label={t('models.oneHourCache')} value={firstOneHourCachePrice(pricing)} />
      </Grid>
    </Grid>
  );
}

function TieredPricing({ tiers }: { tiers: PricingTier[] }) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      {tiers.map((tier, index) => (
        <Stack key={`${tier.up_to ?? 'open'}-${index}`} spacing={1} sx={panelSx}>
          <Typography variant="subtitle2">{tierLabel({ t, tiers, tier, index })}</Typography>
          <Grid container spacing={1}>
            <Grid size={{ xs: 12 }}>
              <PriceBox label={t('fields.inputPrice')} value={`$${tier.input_price_per_1m.toFixed(2)}`} />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox label={t('fields.outputPrice')} value={`$${tier.output_price_per_1m.toFixed(2)}`} />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox label={t('fields.cacheCreationPrice')} value={nullablePrice(tier.cache_creation_price_per_1m)} />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <PriceBox label={t('fields.cacheReadPrice')} value={nullablePrice(tier.cache_read_price_per_1m)} />
            </Grid>
            <Grid size={{ xs: 12 }}>
              <InlinePrice label={t('models.oneHourCache')} value={oneHourCachePrice(tier)} />
            </Grid>
          </Grid>
        </Stack>
      ))}
    </Stack>
  );
}

function RequestPrice({ value }: { value?: number | null }) {
  const { t } = useTranslate('admin');
  const price = requestPrice(value);
  if (!price) return null;
  return <InlinePrice label={t('fields.pricePerRequest')} value={price} />;
}

function PriceBox({ label, value }: { label: string; value: string }) {
  return (
    <Stack spacing={0.75} sx={panelSx}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="subtitle1" sx={{ fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function InlinePrice({ label, value }: { label: string; value: string }) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between" spacing={2} sx={panelSx}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
        {value}
      </Typography>
    </Stack>
  );
}

function copyModelName(copy: (value: string) => void, name: string, message: string) {
  copy(name);
  toast.success(message);
}

function tierLabel({
  t,
  tiers,
  tier,
  index,
}: {
  t: (key: string) => string;
  tiers: PricingTier[];
  tier: PricingTier;
  index: number;
}) {
  if (tier.up_to === null) return index === 0 ? t('models.tierAll') : `> ${formatTierLimit(tiers[index - 1]?.up_to)}`;
  const start = index === 0 ? '0' : formatTierLimit(tiers[index - 1]?.up_to);
  return `${start} - ${formatTierLimit(tier.up_to)}`;
}

function formatTierLimit(limit?: number | null) {
  if (!limit) return '0';
  if (limit >= 1000000) return `${(limit / 1000000).toFixed(1)}M`;
  if (limit >= 1000) return `${(limit / 1000).toFixed(0)}K`;
  return String(limit);
}

function nullablePrice(value?: number | null) {
  return value === null || value === undefined ? '-' : `$${value.toFixed(2)}`;
}

function drawerSlotProps(onExited: () => void) {
  return {
    backdrop: { invisible: true },
    paper: {
      sx: [
        (theme: Theme) => ({
          ...theme.mixins.paperStyles(theme, {
            color: varAlpha(theme.vars.palette.background.defaultChannel, 0.9),
          }),
          width: 360,
        }),
      ],
    },
    transition: { onExited },
  };
}

const headerSx = {
  py: 2,
  pr: 1,
  pl: 2.5,
  display: 'flex',
  alignItems: 'center',
};

const contentSx = {
  pb: 5,
  px: 2.5,
};

const modelNameSx = {
  minWidth: 0,
  fontFamily: 'monospace',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
  whiteSpace: 'nowrap',
};

const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const capabilityPanelSx = {
  ...panelSx,
  minHeight: 72,
};
