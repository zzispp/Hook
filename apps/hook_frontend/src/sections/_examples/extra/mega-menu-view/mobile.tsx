import type { MegaMenuMobileProps } from 'src/components/mega-menu';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';

import { Logo } from 'src/components/logo';
import { Iconify } from 'src/components/iconify';
import { MegaMenuMobile } from 'src/components/mega-menu';

import { MEGA_MENU_ITEMS } from './data';

// ----------------------------------------------------------------------

export function DemoMegaMenuMobile({ submenuMode }: Pick<MegaMenuMobileProps, 'submenuMode'>) {
  return (
    <MegaMenuMobile
      submenuMode={submenuMode}
      data={MEGA_MENU_ITEMS}
      cssVars={{ '--nav-item-gap': '8px' }}
      slots={{
        button: (
          <Button
            variant="contained"
            color={submenuMode === 'drawer' ? 'inherit' : 'primary'}
            startIcon={<Iconify icon="carbon:menu" />}
          >
            Mobile menu ({submenuMode})
          </Button>
        ),
        topArea: (
          <Box sx={{ px: 2.5, py: 3 }}>
            <Logo />
          </Box>
        ),
        bottomArea: (
          <Divider>
            <Box
              sx={{
                p: 2,
                textAlign: 'center',
                color: 'text.secondary',
                typography: 'subtitle2',
              }}
            >
              Bottom
            </Box>
          </Divider>
        ),
      }}
      slotProps={{ rootItem: { sx: {} }, subItem: {} }}
    />
  );
}
