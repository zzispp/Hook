import type { TFunction } from 'i18next';
import type { AdminAffiliateUserSummary } from 'src/types/affiliate';

export function formatMoney(value: number, locale: string) {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6,
  }).format(value);
}

export function formatPercent(value: number, locale: string) {
  return `${new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 4,
  }).format(value)}%`;
}

export function formatCount(value: number, locale: string) {
  return new Intl.NumberFormat(locale).format(value);
}

export function formatDate(value: string | null | undefined, locale: string) {
  if (!value) return '-';
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(locale, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
}

export function userLabel(user: AdminAffiliateUserSummary | null | undefined, t: TFunction<'admin'>) {
  if (!user) return '-';
  return user.username || user.email || user.id;
}
