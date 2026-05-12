'use client';

import type { Theme } from '@mui/material/styles';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { ModelCopyButton } from './model-copy-button';
import { ModelPricingSection } from './model-pricing-section';
import { ModelGroupPricingSection } from './model-group-pricing-section';
import { hasCapability, MODEL_DETAIL_CAPABILITIES } from './model-catalog-utils';

// ----------------------------------------------------------------------

type Props = {
  model: GlobalModelResponse | null;
  groups: BillingGroup[];
  groupsLoading: boolean;
  groupsErrorMessage?: string;
  currencyDisplay?: CurrencyDisplay;
  open: boolean;
  onClose: () => void;
  onExited: () => void;
};

export function ModelDetailDrawer({
  model,
  groups,
  groupsLoading,
  groupsErrorMessage,
  currencyDisplay,
  open,
  onClose,
  onExited,
}: Props) {
  return (
    <Drawer anchor="right" open={open} onClose={onClose} slotProps={drawerSlotProps(onExited)}>
      {model ? (
        <>
          <DrawerHeader onClose={onClose} />
          <Scrollbar>
            <Stack spacing={3} sx={contentSx}>
              <ModelSummary model={model} />
              <CapabilitySection model={model} />
              <ModelPricingSection model={model} currencyDisplay={currencyDisplay} />
              <ModelGroupPricingSection
                model={model}
                groups={groups}
                loading={groupsLoading}
                errorMessage={groupsErrorMessage}
                currencyDisplay={currencyDisplay}
              />
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
        <ModelCopyButton value={model.name} />
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
      <Iconify
        width={22}
        icon={supported ? 'solar:check-circle-bold' : 'solar:forbidden-circle-bold'}
      />
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

export const panelSx = {
  p: 1.5,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.palette.divider}`,
};

const capabilityPanelSx = {
  ...panelSx,
  minHeight: 72,
};
