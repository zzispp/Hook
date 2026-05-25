import type { TFunction } from 'i18next';
import type { LabelColor } from 'src/components/label';

import { formatMoney } from 'src/utils/currency-format';

export function formatUsd(value?: number | null) {
  return formatMoney(value);
}

export function formatCny(value?: number | null) {
  return `¥${Number(value ?? 0).toFixed(2)}`;
}

export function formatRechargeDate(value: string, locale: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(date);
}

export function rechargePackageStatusLabel(t: TFunction<'admin'>, status: string) {
  return t(`adminRecharges.status.package.${status}`);
}

export function rechargeOrderStatusLabel(t: TFunction<'admin'>, status: string) {
  return t(`adminRecharges.status.order.${status}`);
}

export function packageStatusColor(status: string): LabelColor {
  return status === 'active' ? 'success' : 'default';
}

export function orderStatusColor(status: string): LabelColor {
  if (status === 'paid') return 'success';
  if (status === 'pending') return 'warning';
  if (status === 'failed' || status === 'expired') return 'error';
  return 'default';
}

export function estimatedPayableAmount(rechargeAmount: number, ratio: number) {
  return rechargeAmount * ratio;
}

