'use client';

import type { TFunction } from 'i18next';
import type { RechargeOrder } from 'src/types/recharge';
import type { WalletLedgerExpansionState } from './wallet-ledger-entries-table';
import type { WalletFilterOption, WalletLedgerFilterState } from './wallet-filters';
import type { WalletSummary, WalletLedgerEntry, WalletTransaction } from 'src/types/wallet';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { WalletLedgerFilters } from './wallet-ledger-filters';
import { WalletRechargeOrdersTab } from './wallet-recharge-orders-tab';
import { WalletLedgerEntriesTable } from './wallet-ledger-entries-table';

type WalletLedgerTab = 'ledger' | 'orders';

type Props = {
  t: TFunction<'admin'>;
  wallet?: WalletSummary;
  locale: string;
  rechargeOrders: RechargeOrder[];
  loading: boolean;
  filters: WalletLedgerFilterState;
  hasFilters: boolean;
  reasonOptions: WalletFilterOption[];
  linkTypeOptions: WalletFilterOption[];
  items: WalletLedgerEntry[];
  total: number;
  loadedCount: number;
  page: number;
  rowsPerPage: number;
  expansion: WalletLedgerExpansionState;
  onFilterChange: (filters: WalletLedgerFilterState) => void;
  onOpen: (transaction: WalletTransaction) => void;
  onToggleDailyUsage: (entry: WalletLedgerEntry) => void;
  onDailyUsagePageChange: (entry: WalletLedgerEntry, page: number, pageSize: number) => void;
  onPageChange: (event: unknown, newPage: number) => void;
  onRowsPerPageChange: React.ChangeEventHandler<HTMLInputElement>;
};

export function WalletLedgerSection(props: Props) {
  const [tab, setTab] = useState<WalletLedgerTab>('ledger');

  return (
    <Card>
      <Stack spacing={2} sx={{ px: 2.5, pt: 2.5, pb: 2 }}>
        <Typography variant="h6">{props.t('wallet.tabs.ledger')}</Typography>
        <Tabs value={tab} onChange={(_, value: WalletLedgerTab) => setTab(value)}>
          <Tab value="ledger" label={props.t('wallet.tabs.ledger')} />
          <Tab value="orders" label={props.t('wallet.recharge.ordersTitle')} />
        </Tabs>
      </Stack>
      {tab === 'ledger' ? <WalletLedgerTabPanel {...props} /> : null}
      {tab === 'orders' ? <WalletRechargeOrdersTab t={props.t} locale={props.locale} orders={props.rechargeOrders} /> : null}
    </Card>
  );
}

function WalletLedgerTabPanel(props: Props) {
  return (
    <>
      <WalletLedgerFilters
        t={props.t}
        filters={props.filters}
        hasFilters={props.hasFilters}
        reasonOptions={props.reasonOptions}
        linkTypeOptions={props.linkTypeOptions}
        onChange={props.onFilterChange}
      />
      <WalletLedgerEntriesTable
        t={props.t}
        wallet={props.wallet}
        locale={props.locale}
        loading={props.loading}
        items={props.items}
        total={props.total}
        loadedCount={props.loadedCount}
        page={props.page}
        rowsPerPage={props.rowsPerPage}
        expansion={props.expansion}
        onOpen={props.onOpen}
        onToggleDailyUsage={props.onToggleDailyUsage}
        onDailyUsagePageChange={props.onDailyUsagePageChange}
        onPageChange={props.onPageChange}
        onRowsPerPageChange={props.onRowsPerPageChange}
      />
    </>
  );
}
