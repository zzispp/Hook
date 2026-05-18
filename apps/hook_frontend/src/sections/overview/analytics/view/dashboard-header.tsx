import type { TFunction } from 'i18next';
import type { DashboardPreset } from 'src/types/dashboard';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import ButtonGroup from '@mui/material/ButtonGroup';

import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

const PRESETS: DashboardPreset[] = ['today', '7d', '30d', '90d'];

export function DashboardHeader({
  t,
  preset,
  loading,
  isAdmin,
  onRefresh,
  onPresetChange,
}: {
  t: TFunction<'admin'>;
  preset: DashboardPreset;
  loading: boolean;
  isAdmin: boolean;
  onRefresh: VoidFunction;
  onPresetChange: (preset: DashboardPreset) => void;
}) {
  const breadcrumbs = useDashboardBreadcrumbs({ headingCode: DASHBOARD_MENU_CODES.dashboard });
  const subtitle = isAdmin ? t('dashboard.stats.subtitleAdmin') : t('dashboard.stats.subtitleUser');

  return (
    <Stack spacing={1} sx={{ mb: { xs: 3, md: 5 } }}>
      <CustomBreadcrumbs
        heading={t('dashboard.stats.title')}
        links={breadcrumbs.links}
        action={
          <Stack direction="row" spacing={1} alignItems="center">
            <ButtonGroup variant="outlined" size="small">
              {PRESETS.map((item) => (
                <Button
                  key={item}
                  variant={preset === item ? 'contained' : 'outlined'}
                  onClick={() => onPresetChange(item)}
                >
                  {t(`dashboard.stats.presets.${item}`)}
                </Button>
              ))}
            </ButtonGroup>
            <Button
              color="inherit"
              variant="outlined"
              loading={loading}
              startIcon={<Iconify icon="solar:restart-bold" />}
              onClick={onRefresh}
            >
              {t('common.refresh')}
            </Button>
          </Stack>
        }
      />
      <Typography variant="body2" sx={{ color: 'text.secondary' }}>
        {subtitle}
      </Typography>
    </Stack>
  );
}
