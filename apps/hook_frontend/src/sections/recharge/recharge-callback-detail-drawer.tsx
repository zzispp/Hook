'use client';

import type { TFunction } from 'i18next';
import type { Theme } from '@mui/material/styles';
import type { PaymentCallbackRecord } from 'src/types/recharge';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import { RequestRecordJsonViewer } from 'src/sections/admin/request-record-json-viewer';

import {
  formatRechargeDate,
  callbackStatusColor,
  paymentCallbackStatusLabel,
} from './recharge-display';

type Props = {
  t: TFunction<'admin'>;
  open: boolean;
  record: PaymentCallbackRecord | null;
  locale: string;
  onClose: VoidFunction;
};

export function RechargeCallbackDetailDrawer({ t, open, record, locale, onClose }: Props) {
  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      disableScrollLock
      disableRestoreFocus
      slotProps={drawerSlotProps}
    >
      <DrawerHeader t={t} record={record} locale={locale} onClose={onClose} />
      <Scrollbar>
        <Stack spacing={2.5} sx={{ px: 2.5, pb: 5 }}>
          <CallbackMetaPanel t={t} record={record} locale={locale} />
          <CallbackErrorPanel t={t} record={record} />
          <CallbackRawParamsPanel t={t} record={record} />
        </Stack>
      </Scrollbar>
    </Drawer>
  );
}

function DrawerHeader({
  t,
  record,
  locale,
  onClose,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord | null;
  locale: string;
  onClose: VoidFunction;
}) {
  return (
    <Box sx={headerSx}>
      <Stack spacing={1} sx={{ flexGrow: 1, minWidth: 0 }}>
        <Stack direction="row" spacing={1} alignItems="center" useFlexGap flexWrap="wrap">
          <Typography variant="h6">{t('adminRecharges.tabs.callbacks')}</Typography>
          {record ? <CallbackStatusLabel t={t} record={record} /> : null}
        </Stack>
        {record ? <HeaderMeta t={t} record={record} locale={locale} /> : null}
      </Stack>
      <Tooltip title={t('common.close')}>
        <IconButton onClick={onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

function HeaderMeta({
  t,
  record,
  locale,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord;
  locale: string;
}) {
  const items = [
    `ID: ${record.id}`,
    formatRechargeDate(record.received_at, locale),
    `${t('adminRecharges.fields.orderNo')}: ${record.order_no || '-'}`,
    `${t('adminRecharges.fields.paymentChannel')}: ${record.payment_channel_code}`,
  ];

  return (
    <Typography variant="caption" color="text.secondary">
      {items.join(' | ')}
    </Typography>
  );
}

function CallbackStatusLabel({
  t,
  record,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord;
}) {
  return (
    <>
      <Label color={callbackStatusColor(record.status)} variant="soft">
        {paymentCallbackStatusLabel(t, record.status)}
      </Label>
      <Label color={record.settled ? 'success' : 'default'} variant="soft">
        {record.settled
          ? t('adminRecharges.settlement.settled')
          : t('adminRecharges.settlement.unsettled')}
      </Label>
    </>
  );
}

function CallbackMetaPanel({
  t,
  record,
  locale,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord | null;
  locale: string;
}) {
  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Stack direction="row" spacing={2} useFlexGap flexWrap="wrap">
        {callbackMetaItems(t, record, locale).map((item) => (
          <Stack key={item.label} spacing={0.25} sx={{ minWidth: 120 }}>
            <Typography variant="caption" color="text.secondary">
              {item.label}
            </Typography>
            <Typography variant="subtitle2">{item.value}</Typography>
          </Stack>
        ))}
      </Stack>
    </Stack>
  );
}

function callbackMetaItems(
  t: TFunction<'admin'>,
  record: PaymentCallbackRecord | null,
  locale: string
) {
  return [
    [t('adminRecharges.fields.callbackKind'), callbackKindLabel(t, record)],
    [t('adminRecharges.fields.httpMethod'), record?.http_method ?? '-'],
    [t('adminRecharges.fields.providerTradeNo'), record?.provider_trade_no || '-'],
    [t('adminRecharges.fields.paymentMethod'), record?.payment_method || '-'],
    [t('adminRecharges.fields.tradeStatus'), record?.trade_status || '-'],
    [t('adminRecharges.fields.processedAt'), callbackProcessedAt(record, locale)],
  ].map(([label, value]) => ({ label, value }));
}

function callbackKindLabel(t: TFunction<'admin'>, record: PaymentCallbackRecord | null) {
  if (!record) return '-';
  return t(`adminRecharges.callbackKind.${record.callback_kind}`);
}

function callbackProcessedAt(record: PaymentCallbackRecord | null, locale: string) {
  if (!record?.processed_at) return '-';
  return formatRechargeDate(record.processed_at, locale);
}

function CallbackErrorPanel({
  t,
  record,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord | null;
}) {
  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Typography variant="subtitle2">{t('adminRecharges.fields.errorMessage')}</Typography>
      <Divider />
      <Typography variant="body2" color={record?.error_message ? 'text.primary' : 'text.secondary'}>
        {record?.error_message || '-'}
      </Typography>
    </Stack>
  );
}

function CallbackRawParamsPanel({
  t,
  record,
}: {
  t: TFunction<'admin'>;
  record: PaymentCallbackRecord | null;
}) {
  return (
    <Stack spacing={1.5} sx={panelSx}>
      <Typography variant="subtitle2">{t('adminRecharges.fields.rawParams')}</Typography>
      <Divider />
      <RequestRecordJsonViewer value={record?.raw_params ?? {}} />
    </Stack>
  );
}

const drawerSlotProps = {
  backdrop: { invisible: true },
  paper: {
    sx: [
      (theme: Theme) => ({
        ...theme.mixins.paperStyles(theme, {
          color: varAlpha(theme.vars.palette.background.defaultChannel, 0.95),
        }),
        width: { xs: 1, sm: 680 },
      }),
    ],
  },
};

const headerSx = {
  py: 2,
  pr: 1,
  pl: 2.5,
  gap: 1,
  display: 'flex',
  alignItems: 'flex-start',
};

const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};
