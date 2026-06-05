'use client';

import type { TFunction } from 'i18next';
import type { AffiliateSummary } from 'src/types/account-affiliate';

import { useMemo, useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';
import { DASHBOARD_MENU_CODES, DASHBOARD_SECTION_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { ReferralsTable, CommissionsTable } from './affiliate-tables';
import { type AffiliateCenterTab, useAffiliateCenterState } from './affiliate-state';
import { ReferralFiltersToolbar, CommissionFiltersToolbar } from './affiliate-filters';
import { formatDate, formatMoney, formatCount, formatPercent } from './affiliate-format';

export function AffiliateCenterView() {
  const { t, currentLang } = useTranslate('admin');
  const [tab, setTab] = useState<AffiliateCenterTab>('referrals');
  const state = useAffiliateCenterState(t);
  const locale = currentLang.numberFormat.code;
  const loading = tab === 'referrals' ? state.referrals.isLoading : state.commissions.isLoading;
  const fullLink = useFullAffiliateLink(state.summary.data);

  return (
    <DashboardContent maxWidth="xl">
      <AffiliateBreadcrumbs
        t={t}
        loading={loading || state.summary.isValidating}
        onRefresh={() => state.refresh(tab)}
      />
      {state.errorMessage ? <ErrorAlert message={state.errorMessage} /> : null}
      <AffiliateDisabledAlert t={t} summary={state.summary.data} />
      <InviteCard t={t} summary={state.summary.data} fullLink={fullLink} />
      <OverviewCards t={t} locale={locale} summary={state.summary.data} />
      <Tabs value={tab} onChange={(_event, next: AffiliateCenterTab) => setTab(next)} sx={{ mb: 3 }}>
        <Tab value="referrals" label={t('affiliateCenter.tabs.referrals')} />
        <Tab value="commissions" label={t('affiliateCenter.tabs.commissions')} />
      </Tabs>
      {tab === 'referrals' ? <ReferralsPanel t={t} locale={locale} state={state} /> : null}
      {tab === 'commissions' ? <CommissionsPanel t={t} locale={locale} state={state} /> : null}
    </DashboardContent>
  );
}

function AffiliateBreadcrumbs({
  t,
  loading,
  onRefresh,
}: {
  t: TFunction<'admin'>;
  loading: boolean;
  onRefresh: VoidFunction;
}) {
  const breadcrumbs = useDashboardBreadcrumbs({
    headingCode: DASHBOARD_MENU_CODES.affiliateCenter,
    sectionCode: DASHBOARD_SECTION_CODES.operations,
  });

  return (
    <CustomBreadcrumbs
      heading={breadcrumbs.heading}
      links={breadcrumbs.links}
      action={
        <Button
          color="inherit"
          variant="outlined"
          loading={loading}
          startIcon={<Iconify icon="solar:restart-bold" />}
          onClick={onRefresh}
        >
          {t('models.refresh')}
        </Button>
      }
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

function AffiliateDisabledAlert({
  t,
  summary,
}: {
  t: TFunction<'admin'>;
  summary?: AffiliateSummary;
}) {
  if (!summary || summary.affiliate_enabled) return null;

  return (
    <Alert severity="warning" sx={{ mb: 3 }}>
      {t('affiliateCenter.messages.affiliateDisabled')}
    </Alert>
  );
}

function InviteCard({
  t,
  summary,
  fullLink,
}: {
  t: TFunction<'admin'>;
  summary?: AffiliateSummary;
  fullLink: string;
}) {
  return (
    <Card sx={{ p: 2.5, mb: 3 }}>
      <Grid container spacing={2} alignItems="center">
        <Grid size={{ xs: 12, md: 3 }}>
          <TextField
            fullWidth
            label={t('affiliateCenter.fields.affiliateCode')}
            value={summary?.affiliate_code ?? ''}
            slotProps={{ input: { readOnly: true } }}
          />
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          <TextField
            fullWidth
            label={t('affiliateCenter.fields.inviteLink')}
            value={fullLink}
            slotProps={{ input: { readOnly: true } }}
          />
        </Grid>
        <Grid size={{ xs: 12, md: 3 }}>
          <Button
            fullWidth
            variant="contained"
            startIcon={<Iconify icon="solar:copy-bold" />}
            disabled={!fullLink}
            onClick={() => copyInviteLink(fullLink, t)}
          >
            {t('affiliateCenter.actions.copyLink')}
          </Button>
        </Grid>
      </Grid>
    </Card>
  );
}

function OverviewCards({
  t,
  locale,
  summary,
}: {
  t: TFunction<'admin'>;
  locale: string;
  summary?: AffiliateSummary;
}) {
  const items = [
    { label: t('affiliateCenter.overview.referredUsers'), value: formatCount(summary?.referred_user_count ?? 0, locale) },
    { label: t('affiliateCenter.overview.referredRecharge'), value: formatMoney(summary?.total_referred_recharge_amount ?? 0, locale) },
    { label: t('affiliateCenter.overview.totalCommission'), value: formatMoney(summary?.total_commission_amount ?? 0, locale) },
    { label: t('affiliateCenter.overview.todayCommission'), value: formatMoney(summary?.today_commission_amount ?? 0, locale) },
    { label: t('affiliateCenter.overview.monthCommission'), value: formatMoney(summary?.month_commission_amount ?? 0, locale) },
    { label: t('affiliateCenter.overview.commissionPercent'), value: formatPercent(summary?.affiliate_commission_percent ?? 0, locale) },
    { label: t('affiliateCenter.overview.lastCommissionAt'), value: formatDate(summary?.last_commission_at, locale) },
  ];

  return (
    <Grid container spacing={2} sx={{ mb: 3 }}>
      {items.map((item) => (
        <Grid key={item.label} size={{ xs: 12, sm: 6, md: 4, lg: item.label.length > 18 ? 3 : undefined }}>
          <Card sx={{ p: 2 }}>
            <Typography variant="caption" color="text.secondary">
              {item.label}
            </Typography>
            <Typography variant="h6" sx={{ mt: 0.5 }}>
              {item.value}
            </Typography>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}

function ReferralsPanel({ t, locale, state }: PanelProps) {
  return (
    <Card>
      <ReferralFiltersToolbar t={t} filters={state.referralFilters} onChange={state.changeReferralFilters} />
      <ReferralsTable
        t={t}
        locale={locale}
        rows={state.referrals.data?.items ?? []}
        total={state.referrals.data?.total ?? 0}
        loading={state.referrals.isLoading}
        page={state.referralTable.page}
        rowsPerPage={state.referralTable.rowsPerPage}
        onPageChange={state.referralTable.onChangePage}
        onRowsPerPageChange={state.referralTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function CommissionsPanel({ t, locale, state }: PanelProps) {
  return (
    <Card>
      <CommissionFiltersToolbar
        t={t}
        filters={state.commissionFilters}
        onChange={state.changeCommissionFilters}
        onExport={state.exportCommissions}
      />
      <CommissionsTable
        t={t}
        locale={locale}
        rows={state.commissions.data?.items ?? []}
        total={state.commissions.data?.total ?? 0}
        loading={state.commissions.isLoading}
        page={state.commissionTable.page}
        rowsPerPage={state.commissionTable.rowsPerPage}
        onPageChange={state.commissionTable.onChangePage}
        onRowsPerPageChange={state.commissionTable.onChangeRowsPerPage}
      />
    </Card>
  );
}

function ErrorAlert({ message }: { message: string }) {
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

function useFullAffiliateLink(summary?: AffiliateSummary) {
  return useMemo(() => {
    if (!summary?.affiliate_link || typeof window === 'undefined') return summary?.affiliate_link ?? '';
    return new URL(summary.affiliate_link, window.location.origin).toString();
  }, [summary?.affiliate_link]);
}

async function copyInviteLink(link: string, t: TFunction<'admin'>) {
  await navigator.clipboard.writeText(link);
  toast.success(t('affiliateCenter.messages.linkCopied'));
}

type PanelProps = {
  t: TFunction<'admin'>;
  locale: string;
  state: ReturnType<typeof useAffiliateCenterState>;
};
