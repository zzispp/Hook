'use client';

import Box from '@mui/material/Box';

import { DemoMegaMenuMobile } from './mobile';
import { ComponentLayout } from '../../layout';
import { DemoMegaMenuVertical } from './vertical';
import { DemoMegaMenuHorizontal } from './horizontal';

// ----------------------------------------------------------------------

export function MegaMenuView() {
  return (
    <ComponentLayout
      heroProps={{
        heading: 'Mega menu',
        bottomNode: <DemoMegaMenuHorizontal />,
      }}
      containerProps={{ maxWidth: 'lg' }}
    >
      <Box sx={{ mb: 5, gap: 1.5, display: 'flex' }}>
        <DemoMegaMenuMobile submenuMode="drawer" />
        <DemoMegaMenuMobile submenuMode="collapse" />
      </Box>
      <DemoMegaMenuVertical />
    </ComponentLayout>
  );
}
