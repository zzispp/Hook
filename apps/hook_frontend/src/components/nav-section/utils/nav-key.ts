import type { NavSectionProps, NavItemDataProps } from '../types';

type NavGroupData = NavSectionProps['data'][number];

export function navGroupKey(group: NavGroupData) {
  return group.code ?? group.subheader ?? navItemKey(group.items[0]);
}

export function navItemKey(item: NavItemDataProps) {
  return item.code ?? item.path ?? item.title;
}
