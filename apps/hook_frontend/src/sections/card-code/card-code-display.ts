import type { TFunction } from 'i18next';
import type { LabelColor } from 'src/components/label';
import type { CardCodeStatus, CardCodeBalanceType } from 'src/types/card-code';

import { formatWalletMoney, formatWalletDateTime } from 'src/sections/wallet/wallet-display';

export function cardCodeStatusLabel(t: TFunction<'admin'>, status: string) {
  return t(`adminCardCodes.status.${status}`);
}

export function cardCodeStatusColor(status: string): LabelColor {
  if (status === 'active') return 'success';
  if (status === 'disabled') return 'warning';
  if (status === 'used') return 'info';
  return 'default';
}

export function cardCodeTypeStatusColor(status: CardCodeStatus): LabelColor {
  return status === 'active' ? 'success' : 'default';
}

export function cardCodeBalanceTypeLabel(t: TFunction<'admin'>, balanceType: CardCodeBalanceType) {
  return t(`wallet.balanceTypeLabels.${balanceType}`);
}

export function formatCardCodeAmount(recharge: number, gift: number) {
  return `${formatWalletMoney(recharge, 'CNY')} / ${formatWalletMoney(gift, 'CNY')}`;
}

export function formatCardCodeDate(value: string | null | undefined, locale: string) {
  return value ? formatWalletDateTime(value, locale) : '-';
}
