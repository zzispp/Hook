import type { NavMainProps } from './main/nav/types';

import { paths } from 'src/routes/paths';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

export const navData: NavMainProps['data'] = [
  {
    title: 'Home',
    titleKey: 'nav.home',
    path: '/',
    icon: <Iconify width={22} icon="solar:home-angle-bold-duotone" />,
  },
  {
    title: 'Dashboard',
    titleKey: 'nav.dashboard',
    path: paths.dashboard.root,
    icon: <Iconify width={22} icon="solar:file-bold-duotone" />,
  },
];
