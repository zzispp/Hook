'use client';

import type { GlobalModelResponse } from 'src/types/model';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import InputAdornment from '@mui/material/InputAdornment';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import {
  DASHBOARD_MENU_TITLES,
  DASHBOARD_MENU_SECTIONS,
} from 'src/layouts/dashboard/dashboard-menu-values';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import { useUserModelCatalog } from 'src/actions/models';
import { useAvailableBillingGroups } from 'src/actions/groups';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { ModelCatalogCards } from './model-catalog-cards';
import { ModelCatalogTable } from './model-catalog-table';
import { ModelDetailDrawer } from './model-detail-drawer';
import { filterCatalogItems } from './model-catalog-utils';

// ----------------------------------------------------------------------

export function ModelCatalogView() {
  const { t } = useTranslate('admin');
  const catalog = useUserModelCatalog();
  const groups = useAvailableBillingGroups();
  const breadcrumbs = useDashboardBreadcrumbs({
    heading: DASHBOARD_MENU_TITLES.modelCatalog,
    section: DASHBOARD_MENU_SECTIONS.operations,
  });
  const [query, setQuery] = useState('');
  const [drawerOpen, setDrawerOpen] = useState(false);
  const [selectedModel, setSelectedModel] = useState<GlobalModelResponse | null>(null);
  const rows = useMemo(() => filterCatalogItems(catalog.items, query), [catalog.items, query]);
  const handleSelectModel = useCallback((model: GlobalModelResponse) => {
    setSelectedModel(model);
    setDrawerOpen(true);
  }, []);

  const handleCloseDrawer = useCallback(() => {
    setDrawerOpen(false);
  }, []);

  return (
    <DashboardContent maxWidth="xl">
      <CustomBreadcrumbs
        heading={breadcrumbs.heading}
        links={breadcrumbs.links}
        sx={{ mb: { xs: 3, md: 5 } }}
      />

      <Card>
        <CatalogToolbar
          query={query}
          total={catalog.total}
          refreshing={catalog.isValidating}
          onQueryChange={setQuery}
          onRefresh={() => catalog.refresh()}
        />
        {catalog.error ? <ErrorState message={catalog.error.message} /> : null}
        <Stack sx={{ display: { xs: 'none', md: 'block' } }}>
          <ModelCatalogTable rows={rows} loading={catalog.isLoading} onSelectRow={handleSelectModel} />
        </Stack>
        <Stack sx={{ display: { xs: 'flex', md: 'none' }, p: 2 }}>
          <ModelCatalogCards rows={rows} onSelectRow={handleSelectModel} />
          {!catalog.isLoading && rows.length === 0 && (
            <Typography variant="body2" color="text.secondary">
              {t('models.emptyCatalog')}
            </Typography>
          )}
        </Stack>
      </Card>
      <ModelDetailDrawer
        model={selectedModel}
        groups={groups.items}
        groupsLoading={groups.isLoading}
        groupsErrorMessage={groups.error?.message}
        open={drawerOpen}
        onClose={handleCloseDrawer}
        onExited={() => setSelectedModel(null)}
      />
    </DashboardContent>
  );
}

function CatalogToolbar({
  query,
  total,
  refreshing,
  onQueryChange,
  onRefresh,
}: {
  query: string;
  total: number;
  refreshing: boolean;
  onQueryChange: (value: string) => void;
  onRefresh: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack
      spacing={2}
      direction={{ xs: 'column', md: 'row' }}
      alignItems={{ xs: 'stretch', md: 'center' }}
      justifyContent="space-between"
      sx={{ p: 2.5 }}
    >
      <Stack spacing={0.5}>
        <Typography variant="h6">{t('models.availableModels')}</Typography>
        <Typography variant="body2" color="text.secondary">
          {t('models.modelsTotal', { count: total })}
        </Typography>
      </Stack>
      <Stack direction="row" spacing={1.5} sx={{ minWidth: 0 }}>
        <TextField
          size="small"
          value={query}
          placeholder={t('models.searchPlaceholder')}
          onChange={(event) => onQueryChange(event.target.value)}
          slotProps={{ input: { startAdornment: <SearchAdornment /> } }}
          sx={{ minWidth: 0, flexGrow: 1, width: { xs: 1, sm: 280 } }}
        />
        <Button
          variant="outlined"
          loading={refreshing}
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
          sx={{ flexShrink: 0, whiteSpace: 'nowrap' }}
        >
          {t('models.refresh')}
        </Button>
      </Stack>
    </Stack>
  );
}

function SearchAdornment() {
  return (
    <InputAdornment position="start">
      <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
    </InputAdornment>
  );
}

function ErrorState({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mx: 2.5, mb: 2 }}>
      {message}
    </Alert>
  );
}
