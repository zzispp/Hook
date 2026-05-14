import type { TFunction } from 'i18next';
import type { CardCode } from 'src/types/card-code';
import type { CardCodeFilters } from 'src/actions/card-code';

import { fetchCardCodes } from 'src/actions/card-code';

import { CARD_CODE_MAX_PAGE_SIZE } from './card-code-constants';

export async function exportableCardCodes(
  selected: CardCode[],
  filters: CardCodeFilters
): Promise<CardCode[]> {
  if (selected.length > 0) {
    return selected;
  }
  return fetchAllCardCodes(filters);
}

export function downloadCardCodesCsv(t: TFunction<'admin'>, items: CardCode[]) {
  const rows = [csvHeaders(t), ...items.map(csvRow)];
  downloadFile('card-codes.csv', rows.map((row) => row.map(csvCell).join(',')).join('\n'), 'text/csv');
}

export function downloadCardCodesTxt(items: CardCode[]) {
  downloadFile('card-codes.txt', items.map((item) => item.code).join('\n'), 'text/plain');
}

async function fetchAllCardCodes(filters: CardCodeFilters) {
  const items: CardCode[] = [];
  let page = 0;
  let total = 0;
  do {
    const response = await fetchCardCodes(page, CARD_CODE_MAX_PAGE_SIZE, filters);
    items.push(...response.items);
    total = response.total;
    page += 1;
  } while (items.length < total);
  return items;
}

function csvHeaders(t: TFunction<'admin'>) {
  return [
    t('adminCardCodes.fields.code'),
    t('common.status'),
    t('adminCardCodes.fields.type'),
    t('wallet.metrics.rechargeBalance'),
    t('wallet.metrics.giftBalance'),
    t('adminCardCodes.fields.usedBy'),
    t('adminCardCodes.fields.usedIp'),
    t('adminCardCodes.fields.usedAt'),
    t('adminCardCodes.fields.expiresAt'),
    t('adminCardCodes.fields.createdAt'),
    t('adminCardCodes.fields.createdIp'),
    t('adminCardCodes.fields.createdBy'),
    t('adminCardCodes.fields.remark'),
  ];
}

function csvRow(item: CardCode) {
  return [
    item.code,
    item.status,
    item.type_name,
    item.recharge_amount,
    item.gift_amount,
    item.used_by_username ?? '',
    item.used_ip ?? '',
    item.used_at ?? '',
    item.expires_at ?? '',
    item.created_at,
    item.created_ip ?? '',
    item.created_by_username ?? '',
    item.remark ?? '',
  ];
}

function csvCell(value: string | number) {
  const text = String(value);
  return /[",\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

function downloadFile(filename: string, content: string, type: string) {
  const blob = new Blob([content], { type: `${type};charset=utf-8` });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}
