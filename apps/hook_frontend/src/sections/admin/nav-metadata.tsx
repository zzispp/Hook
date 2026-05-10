'use client';

import type { NavSectionProps } from 'src/components/nav-section';

import { CONFIG } from 'src/global-config';

import { SvgColor } from 'src/components/svg-color';

// ----------------------------------------------------------------------

export const NAV_ICON_OPTIONS = [
  'icon.dashboard',
  'icon.user',
  'icon.lock',
  'icon.menu',
  'icon.model',
  'icon.settings',
  'icon.wallet',
  'icon.analytics',
  'icon.file',
  'icon.folder',
  'icon.group',
  'icon.key',
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
  'icon.group': icon('ic-menu-item'),
  'icon.kanban': icon('ic-kanban'),
  'icon.key': icon('ic-lock'),
  'icon.lock': icon('ic-lock'),
  'icon.mail': icon('ic-mail'),
  'icon.menu': icon('ic-menu-item'),
  'icon.model': icon('ic-model'),
  'icon.settings': icon('ic-menu-item'),
  'icon.user': icon('ic-user'),
  'icon.wallet': icon('ic-banking'),
};

function icon(name: string) {
  return <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />;
}
