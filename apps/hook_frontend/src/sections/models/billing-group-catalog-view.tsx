'use client';

import type { BillingGroup } from 'src/types/group';

import { useState } from 'react';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { useUserModelCatalog } from 'src/actions/models';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAvailableBillingGroups } from 'src/actions/groups';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import {
  DASHBOARD_MENU_CODES,
  DASHBOARD_SECTION_CODES,
} from 'src/layouts/dashboard/dashboard-menu-values';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { BillingGroupDetailDialog } from './billing-group-detail-dialog';
import { ModelAvailableBillingGroupsSection } from './model-available-billing-groups-section';

export function BillingGroupCatalogView() {
  const { t } = useTranslate('admin');
  const groups = useAvailableBillingGroups();
  const models = useUserModelCatalog();
  const breadcrumbs = useDashboardBreadcrumbs({
    headingCode: DASHBOARD_MENU_CODES.billingGroupCatalog,
    sectionCode: DASHBOARD_SECTION_CODES.operations,
  });
  const [detailTarget, setDetailTarget] = useState<BillingGroup | null>(null);
  const errorMessage = groups.error?.message ?? models.error?.message;
  const loading = groups.isLoading || models.isLoading;

  return (
    <DashboardContent maxWidth="xl">
      <CustomBreadcrumbs
        heading={breadcrumbs.heading}
        links={breadcrumbs.links}
        sx={{ mb: { xs: 3, md: 5 } }}
      />

      <Card>
        <Stack
          spacing={2}
          direction={{ xs: 'column', md: 'row' }}
          alignItems={{ xs: 'stretch', md: 'center' }}
          justifyContent="space-between"
          sx={{ p: 2.5 }}
        >
          <Stack spacing={0.5}>
            <Typography variant="h6">{t('models.visibleGroups')}</Typography>
            <Typography variant="body2" color="text.secondary">
              {t('fields.allowedModels')} / {t('fields.billingMultiplier')}
            </Typography>
          </Stack>
          <Button
            variant="outlined"
            loading={groups.isValidating || models.isValidating}
            startIcon={<Iconify icon="solar:restart-bold" />}
            onClick={() => {
              void groups.refresh();
              void models.refresh();
            }}
            sx={{ flexShrink: 0, whiteSpace: 'nowrap' }}
          >
            {t('models.refresh')}
          </Button>
        </Stack>

        {errorMessage ? (
          <Alert severity="error" sx={{ mx: 2.5, mb: 2 }}>
            {errorMessage}
          </Alert>
        ) : null}

        <Stack sx={{ px: 2.5, pb: 2.5 }}>
          <ModelAvailableBillingGroupsSection
            groups={groups.items}
            models={models.items}
            loading={loading}
            errorMessage={errorMessage}
            onView={setDetailTarget}
          />
        </Stack>
      </Card>
      <BillingGroupDetailDialog
        group={detailTarget}
        models={models.items}
        open={!!detailTarget}
        onClose={() => setDetailTarget(null)}
      />
    </DashboardContent>
  );
}
