import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/global-config';
import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { varFade, varScale, MotionViewport } from 'src/components/animate';

import { SectionTitle } from './components/section-title';
import { FloatLine, FloatDotIcon } from './components/svg-elements';

// ----------------------------------------------------------------------

const PROVIDERS = [
  { key: 'openai' },
  { key: 'claude' },
  { key: 'gemini' },
  { key: 'custom' },
] as const;

const ROUTING_SIGNAL_KEYS = ['health', 'priority', 'modelMapping', 'formatRewrite'] as const;

type TFunction = ReturnType<typeof useTranslate>['t'];

export function HomeProviderMesh({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');

  return (
    <Box component="section" sx={[sectionSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <MotionViewport>
        <DecorativeLines />

        <Container>
          <Grid container spacing={{ xs: 6, md: 10 }} sx={{ alignItems: 'center' }}>
            <Grid size={{ xs: 12, md: 5 }}>
              <SectionTitle
                caption={t('home.providers.caption')}
                title={t('home.providers.title')}
                txtGradient={t('home.providers.gradient')}
                description={t('home.providers.description')}
                sx={{ textAlign: { xs: 'center', md: 'left' } }}
              />
              <SignalList t={t} />
            </Grid>

            <Grid size={{ xs: 12, md: 7 }}>
              <ProviderPanel t={t} />
            </Grid>
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
      <Stack spacing={8} alignItems="center" sx={dotStackSx}>
        <FloatDotIcon />
        <FloatDotIcon sx={{ opacity: 0.24, width: 14, height: 14 }} />
        <Box sx={{ flexGrow: 1 }} />
        <FloatDotIcon sx={{ opacity: 0.24, width: 14, height: 14 }} />
        <FloatDotIcon />
      </Stack>
      <FloatLine vertical sx={{ top: 0, left: 80 }} />
    </>
  );
}

function SignalList({ t }: { t: TFunction }) {
  return (
    <Stack spacing={2} sx={{ mt: 5, maxWidth: 480, mx: { xs: 'auto', md: 0 } }}>
      {ROUTING_SIGNAL_KEYS.map((signal) => (
        <Box key={signal} component={m.div} variants={varFade('inUp', { distance: 24 })} sx={signalSx}>
          <Iconify width={18} icon="eva:checkmark-fill" />
          <Typography variant="body2">{t(`home.providers.signals.${signal}`)}</Typography>
        </Box>
      ))}
    </Stack>
  );
}

function ProviderPanel({ t }: { t: TFunction }) {
  return (
    <Box component={m.div} variants={varScale('in')} sx={panelSx}>
      <Box component="img" alt="Gateway integration" src={integrationImage} sx={imageSx} />

      <Grid container spacing={2}>
        {PROVIDERS.map((provider) => (
          <Grid key={provider.key} size={{ xs: 12, sm: 6 }}>
            <ProviderItem provider={provider} t={t} />
          </Grid>
        ))}
      </Grid>
    </Box>
  );
}

type ProviderItemProps = {
  t: TFunction;
  provider: (typeof PROVIDERS)[number];
};

function ProviderItem({ provider, t }: ProviderItemProps) {
  return (
    <Box sx={providerSx}>
      <Stack direction="row" spacing={1.5} sx={{ alignItems: 'center' }}>
        <Iconify width={22} icon="solar:ssd-round-bold" />
        <Typography variant="h6">{t(`home.providers.items.${provider.key}.name`)}</Typography>
      </Stack>
      <Typography variant="body2" sx={{ mt: 1, color: 'text.secondary' }}>
        {t(`home.providers.items.${provider.key}.note`)}
      </Typography>
    </Box>
  );
}

// ----------------------------------------------------------------------

const integrationImage = `${CONFIG.assetsDir}/assets/illustrations/illustration-integration.webp`;

const sectionSx = {
  position: 'relative',
  py: { xs: 10, md: 16 },
  bgcolor: 'background.neutral',
};

const dotStackSx = {
  top: 64,
  left: 80,
  zIndex: 2,
  bottom: 64,
  position: 'absolute',
  transform: 'translateX(-50%)',
  '& span': { position: 'static', opacity: 0.12 },
};

const signalSx: SxProps<Theme> = [
  (theme) => ({
    gap: 1.5,
    p: 1.5,
    display: 'flex',
    borderRadius: 1.5,
    alignItems: 'center',
    bgcolor: varAlpha(theme.vars.palette.background.paperChannel, 0.64),
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.12)}`,
  }),
];

const panelSx: SxProps<Theme> = [
  (theme) => ({
    p: { xs: 2.5, md: 4 },
    borderRadius: 2,
    bgcolor: 'background.default',
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.16)}`,
    boxShadow: `0 32px 80px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.12)}`,
  }),
];

const imageSx = {
  width: 1,
  maxHeight: 360,
  objectFit: 'contain',
  mb: { xs: 3, md: 4 },
};

const providerSx: SxProps<Theme> = [
  (theme) => ({
    p: 2.5,
    height: 1,
    borderRadius: 1.5,
    bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.06),
  }),
];
