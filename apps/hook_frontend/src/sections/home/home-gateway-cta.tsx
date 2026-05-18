import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { varFade, MotionViewport } from 'src/components/animate';

import { FloatLine, FloatPlusIcon } from './components/svg-elements';

// ----------------------------------------------------------------------

export function HomeGatewayCTA({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');

  return (
    <Box component="section" sx={[sectionSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <MotionViewport>
        <DecorativeLines />

        <Container sx={{ position: 'relative', zIndex: 9 }}>
          <Box sx={panelSx}>
            <Stack spacing={3} sx={{ maxWidth: 720 }}>
              <Typography component={m.h2} variants={varFade('inDown', { distance: 24 })} sx={titleSx}>
                {t('home.cta.title')}
              </Typography>

              <Typography component={m.p} variants={varFade('inUp', { distance: 24 })} sx={descriptionSx}>
                {t('home.cta.description')}
              </Typography>
            </Stack>

            <Box component={m.div} variants={varFade('inRight', { distance: 24 })} sx={buttonWrapSx}>
              <Button
                component={RouterLink}
                color="primary"
                size="large"
                variant="contained"
                href={paths.dashboard.root}
                endIcon={<Iconify icon="eva:arrow-ios-forward-fill" />}
              >
                {t('home.cta.action')}
              </Button>
            </Box>
          </Box>
        </Container>
      </MotionViewport>
    </Box>
  );
}

// ----------------------------------------------------------------------

function DecorativeLines() {
  return (
    <>
      <FloatPlusIcon sx={{ left: 72, top: '50%', mt: -1 }} />
      <FloatLine vertical sx={{ top: 0, left: 80, height: 'calc(50% + 64px)' }} />
      <FloatLine sx={{ top: '50%', left: 0 }} />
    </>
  );
}

const sectionSx = {
  position: 'relative',
  pb: { xs: 10, md: 16 },
};

const panelSx: SxProps<Theme> = [
  (theme) => ({
    gap: 4,
    px: { xs: 3, md: 6 },
    py: { xs: 6, md: 8 },
    display: 'flex',
    overflow: 'hidden',
    borderRadius: 2,
    bgcolor: 'grey.900',
    position: 'relative',
    alignItems: { xs: 'flex-start', md: 'center' },
    textAlign: { xs: 'left', md: 'left' },
    flexDirection: { xs: 'column', md: 'row' },
    justifyContent: 'space-between',
    border: `solid 1px ${theme.vars.palette.grey[800]}`,
    backgroundImage: [
      `linear-gradient(0deg, ${varAlpha(theme.vars.palette.grey['500Channel'], 0.04)} 1px, transparent 1px)`,
      `linear-gradient(90deg, ${varAlpha(theme.vars.palette.grey['500Channel'], 0.04)} 1px, transparent 1px)`,
    ].join(','),
    backgroundSize: '36px 36px',
  }),
];

const titleSx = {
  m: 0,
  color: 'common.white',
  typography: { xs: 'h3', md: 'h2' },
};

const descriptionSx = {
  m: 0,
  color: 'grey.400',
  typography: { xs: 'body1', md: 'h6' },
};

const buttonWrapSx = {
  flexShrink: 0,
};
