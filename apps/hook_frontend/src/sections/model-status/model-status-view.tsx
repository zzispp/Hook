'use client';

import type { ModelStatusListFilters } from 'src/types/model-status';
import type { AdminDashboardRangeFilters } from '../admin/dashboard-date-range-picker';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useModelStatusChecks } from 'src/actions/model-status';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ModelStatusToolbar } from './model-status-toolbar';
import { RefreshButton, AdminBreadcrumbs } from '../admin/shared';
import { DashboardDateRangePicker } from '../admin/dashboard-date-range-picker';
import { ModelStatusCard, ModelStatusLoadingSkeleton } from './model-status-card';

export function ModelStatusView() {
  const { t } = useTranslate('admin');
  const [filters, setFilters] = useState<ModelStatusListFilters>({ preset: 'today' });
  const records = useModelStatusChecks(filters);

  const changeRange = (range: AdminDashboardRangeFilters) =>
    setFilters((current) => ({ ...range, search: current.search, api_format: current.api_format }));

  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        headingCode={DASHBOARD_MENU_CODES.modelStatus}
        action={
          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={1}>
            <DashboardDateRangePicker
              t={t}
              filters={filters}
              translationRoot="modelStatus"
              onChange={changeRange}
            />
            <RefreshButton loading={records.isValidating} onClick={() => void records.refresh()} />
          </Stack>
        }
      />
      <Stack spacing={3}>
        {records.error ? <Alert severity="error">{errorMessage(records.error)}</Alert> : null}

        <Card>
          <ModelStatusToolbar filters={filters} t={t} onChange={setFilters} />
        </Card>

        {records.isLoading ? (
          <ModelStatusLoadingSkeleton />
        ) : records.items.length === 0 ? (
          <Box sx={{ py: 10, textAlign: 'center' }}>
            <Typography variant="h6" color="text.secondary">
              {t('modelStatus.empty')}
            </Typography>
          </Box>
        ) : (
          <Grid container spacing={3}>
            {records.items.map((row) => (
              <Grid key={row.id} size={{ xs: 12, md: 6 }}>
                <ModelStatusCard row={row} t={t} />
              </Grid>
            ))}
          </Grid>
        )}
      </Stack>
    </DashboardContent>
  );
}

function errorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Request failed';
}
