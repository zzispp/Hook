'use client';

import type { AdminT } from './shared';
import type { NavSectionProps } from 'src/components/nav-section';
import type { MenuSection, MenuItem as RbacMenuItem } from 'src/types/rbac';

import { CONFIG } from 'src/global-config';

import { SvgColor } from 'src/components/svg-color';

// ----------------------------------------------------------------------

export const NAV_ICON_OPTIONS = [
  'icon.dashboard',
  'icon.user',
  'icon.lock',
  'icon.menu',
  'icon.model',
  'icon.wallet',
  'icon.analytics',
  'icon.file',
  'icon.folder',
  'icon.calendar',
  'icon.kanban',
  'icon.mail',
  'icon.chat',
  'icon.blank',
];

export const NAV_ICONS: NonNullable<NavSectionProps['render']>['navIcon'] = {
  'icon.analytics': icon('ic-analytics'),
  'icon.blank': icon('ic-blank'),
  'icon.calendar': icon('ic-calendar'),
  'icon.chat': icon('ic-chat'),
  'icon.dashboard': icon('ic-dashboard'),
  'icon.file': icon('ic-file'),
  'icon.folder': icon('ic-folder'),
  'icon.kanban': icon('ic-kanban'),
  'icon.lock': icon('ic-lock'),
  'icon.mail': icon('ic-mail'),
  'icon.menu': icon('ic-menu-item'),
  'icon.model': icon('ic-model'),
  'icon.user': icon('ic-user'),
  'icon.wallet': icon('ic-banking'),
};

export function translatedMenuSection(section: MenuSection, t: AdminT) {
  const keyByCode: Record<string, string> = {
    overview: 'nav.overview',
    account: 'nav.account',
    resources: 'nav.resources',
    system_management: 'nav.systemManagement',
  };

  const key = keyByCode[section.code];

  return key ? t(key) : section.subheader;
}

export function translatedMenuItem(item: RbacMenuItem, t: AdminT) {
  const keyByCode: Record<string, string> = {
    dashboard_home: 'nav.dashboard',
    dashboard_models: 'nav.modelCatalog',
    wallet_center: 'nav.walletCenter',
    admin_users: 'nav.users',
    admin_roles: 'nav.roles',
    admin_apis: 'nav.apis',
    admin_menus: 'nav.menus',
    admin_models: 'nav.models',
  };

  const key = keyByCode[item.code];

  return key ? t(key) : item.title;
}

function icon(name: string) {
  return <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />;
}
