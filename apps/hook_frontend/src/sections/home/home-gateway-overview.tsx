import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { varFade, MotionViewport } from 'src/components/animate';

import { SectionTitle } from './components/section-title';
import { FloatLine, FloatPlusIcon } from './components/svg-elements';

// ----------------------------------------------------------------------

const CAPABILITIES = [
  {
    key: 'unifiedEntry',
    icon: 'solar:share-bold',
  },
  {
    key: 'accessControl',
    icon: 'solar:users-group-rounded-bold',
  },
  {
    key: 'quotaControl',
    icon: 'solar:bill-list-bold',
  },
] as const;

const REQUEST_FLOW_KEYS = ['client', 'gateway', 'policy', 'provider'] as const;

type TFunction = ReturnType<typeof useTranslate>['t'];

export function HomeGatewayOverview({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');

  return (
    <Box component="section" sx={[sectionSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <MotionViewport>
        <DecorativeLines />

        <Container>
          <Grid container spacing={{ xs: 5, md: 8 }} sx={{ alignItems: 'center' }}>
            <Grid size={{ xs: 12, md: 5 }}>
              <SectionTitle
                caption={t('home.overview.caption')}
                title={t('home.overview.title')}
                txtGradient={t('home.overview.gradient')}
                description={t('home.overview.description')}
                sx={{ textAlign: { xs: 'center', md: 'left' } }}
              />
            </Grid>

            <Grid size={{ xs: 12, md: 7 }}>
              <GatewayFlow t={t} />
            </Grid>
          </Grid>

          <Grid container spacing={3} sx={{ mt: { xs: 6, md: 10 } }}>
            {CAPABILITIES.map((item) => (
              <Grid key={item.key} size={{ xs: 12, md: 4 }}>
                <CapabilityCard item={item} t={t} />
              </Grid>
            ))}
          </Grid>
        </Container>
      </MotionViewport>
    </Box>
  );
}

// ----------------------------------------------------------------------

function DecorativeLines() {
  return (
    <>
      <FloatPlusIcon sx={{ top: 72, left: 72 }} />
      <FloatLine sx={{ top: 80, left: 0 }} />
      <FloatLine vertical sx={{ top: 0, left: 80 }} />
    </>
  );
}

function GatewayFlow({ t }: { t: TFunction }) {
  return (
    <Box component={m.div} variants={varFade('inUp', { distance: 24 })} sx={flowSx}>
      {REQUEST_FLOW_KEYS.map((step, index) => (
        <Stack key={step} spacing={1.5} sx={{ alignItems: 'center', minWidth: 128 }}>
          <Box sx={flowNodeSx}>
            <Iconify width={24} icon={index === 1 ? 'solar:settings-bold' : 'solar:forward-bold'} />
          </Box>
          <Typography variant="subtitle2" sx={{ textAlign: 'center' }}>
            {t(`home.overview.flow.${step}`)}
          </Typography>
        </Stack>
      ))}
    </Box>
  );
}

type CapabilityCardProps = {
  t: TFunction;
  item: (typeof CAPABILITIES)[number];
};

function CapabilityCard({ item, t }: CapabilityCardProps) {
  return (
    <Box component={m.div} variants={varFade('inUp', { distance: 24 })} sx={cardSx}>
      <Box sx={cardIconSx}>
        <Iconify width={28} icon={item.icon} />
      </Box>

      <Stack spacing={1.5}>
        <Typography variant="h5">{t(`home.overview.capabilities.${item.key}.title`)}</Typography>
        <Typography sx={{ color: 'text.secondary' }}>
          {t(`home.overview.capabilities.${item.key}.description`)}
        </Typography>
      </Stack>
    </Box>
  );
}

// ----------------------------------------------------------------------

const sectionSx = {
  overflow: 'hidden',
  position: 'relative',
  py: { xs: 10, md: 16 },
};

const flowSx: SxProps<Theme> = [
  (theme) => ({
    gap: 2,
    p: { xs: 3, sm: 4 },
    display: 'grid',
    borderRadius: 2,
    alignItems: 'center',
    gridTemplateColumns: { xs: '1fr', sm: 'repeat(4, 1fr)' },
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.16)}`,
    backgroundImage: `linear-gradient(135deg, ${varAlpha(theme.vars.palette.grey['500Channel'], 0.08)}, transparent)`,
  }),
];

const flowNodeSx: SxProps<Theme> = [
  (theme) => ({
    width: 64,
    height: 64,
    display: 'grid',
    borderRadius: '50%',
    placeItems: 'center',
    color: 'primary.main',
    bgcolor: varAlpha(theme.vars.palette.primary.mainChannel, 0.08),
    boxShadow: `0 24px 48px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.12)}`,
  }),
];

const cardSx: SxProps<Theme> = [
  (theme) => ({
    gap: 3,
    height: 1,
    p: { xs: 3, md: 4 },
    display: 'flex',
    borderRadius: 2,
    flexDirection: 'column',
    bgcolor: 'background.paper',
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.16)}`,
    boxShadow: `0 24px 48px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.08)}`,
  }),
];

const cardIconSx: SxProps<Theme> = [
  (theme) => ({
    width: 56,
    height: 56,
    display: 'grid',
    borderRadius: 2,
    placeItems: 'center',
    color: 'primary.main',
    bgcolor: varAlpha(theme.vars.palette.primary.mainChannel, 0.08),
  }),
];
