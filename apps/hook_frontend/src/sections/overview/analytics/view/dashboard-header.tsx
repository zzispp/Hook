import type { TFunction } from 'i18next';
import type { DashboardScopeFilters } from 'src/actions/dashboard';
import type { DashboardPreset, DashboardFilterOptionsResponse } from 'src/types/dashboard';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import ButtonGroup from '@mui/material/ButtonGroup';

import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { DashboardScopeFilter } from './dashboard-scope-filter';

const PRESETS: DashboardPreset[] = ['today', '7d', '30d', '90d'];

export function DashboardHeader({
  t,
  preset,
  loading,
  isAdmin,
  filters,
  filterOptions,
  onRefresh,
  onFiltersChange,
  onPresetChange,
}: {
  t: TFunction<'admin'>;
  preset: DashboardPreset;
  loading: boolean;
  isAdmin: boolean;
  filters: DashboardScopeFilters;
  filterOptions?: DashboardFilterOptionsResponse;
  onRefresh: VoidFunction;
  onFiltersChange: (filters: DashboardScopeFilters) => void;
  onPresetChange: (preset: DashboardPreset) => void;
}) {
  const breadcrumbs = useDashboardBreadcrumbs({ headingCode: DASHBOARD_MENU_CODES.dashboard });
  const subtitle = isAdmin ? t('dashboard.stats.subtitleAdmin') : t('dashboard.stats.subtitleUser');

  return (
    <Stack spacing={1} sx={{ mb: { xs: 3, md: 5 } }}>
      <CustomBreadcrumbs
        heading={t('dashboard.stats.title')}
        links={breadcrumbs.links}
      />
      <Stack
        direction={{ xs: 'column', md: 'row' }}
        spacing={1}
        alignItems={{ xs: 'stretch', md: 'center' }}
        justifyContent="space-between"
      >
        <Typography variant="body2" sx={{ minWidth: 0, color: 'text.secondary' }}>
          {subtitle}
        </Typography>
        <HeaderActions
          t={t}
          preset={preset}
          loading={loading}
          isAdmin={isAdmin}
          filters={filters}
          filterOptions={filterOptions}
          onRefresh={onRefresh}
          onFiltersChange={onFiltersChange}
          onPresetChange={onPresetChange}
        />
      </Stack>
    </Stack>
  );
}

function HeaderActions({
  t,
  preset,
  loading,
  isAdmin,
  filters,
  filterOptions,
  onRefresh,
  onFiltersChange,
  onPresetChange,
}: {
  t: TFunction<'admin'>;
  preset: DashboardPreset;
  loading: boolean;
  isAdmin: boolean;
  filters: DashboardScopeFilters;
  filterOptions?: DashboardFilterOptionsResponse;
  onRefresh: VoidFunction;
  onFiltersChange: (filters: DashboardScopeFilters) => void;
  onPresetChange: (preset: DashboardPreset) => void;
}) {
  return (
    <Stack
      direction={{ xs: 'column', md: 'row' }}
      spacing={1}
      alignItems={{ xs: 'stretch', md: 'center' }}
    >
      <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1} alignItems={{ sm: 'center' }}>
        {isAdmin ? (
          <DashboardScopeFilter
            t={t}
            filters={filters}
            options={filterOptions}
            onChange={onFiltersChange}
          />
        ) : null}
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
      </Stack>
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
  );
}
