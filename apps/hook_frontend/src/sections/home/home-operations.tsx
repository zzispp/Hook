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

const METRICS = [
  { key: 'entry', value: '1' },
  { key: 'access', value: 'User / Group' },
  { key: 'budget', value: 'Token / Cost' },
  { key: 'health', value: 'Latency / Error' },
] as const;

const OPERATIONS = [
  {
    key: 'loadBalancing',
    icon: 'solar:restart-bold',
  },
  {
    key: 'quota',
    icon: 'solar:bill-list-bold-duotone',
  },
  {
    key: 'monitoring',
    icon: 'solar:monitor-bold',
  },
  {
    key: 'adminConfig',
    icon: 'solar:settings-bold-duotone',
  },
] as const;

type TFunction = ReturnType<typeof useTranslate>['t'];

export function HomeOperations({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');

  return (
    <Box component="section" sx={[sectionSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <MotionViewport>
        <DecorativeLines />

        <Container>
          <SectionTitle
            caption={t('home.operations.caption')}
            title={t('home.operations.title')}
            txtGradient={t('home.operations.gradient')}
            description={t('home.operations.description')}
            sx={{ mx: 'auto', maxWidth: 760, textAlign: 'center' }}
          />

          <MetricStrip t={t} />

          <Grid container spacing={3} sx={{ mt: 3 }}>
            {OPERATIONS.map((item) => (
              <Grid key={item.key} size={{ xs: 12, sm: 6, md: 3 }}>
                <OperationCard item={item} t={t} />
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
      <FloatPlusIcon sx={{ top: 72, right: 72 }} />
      <FloatLine sx={{ top: 80, left: 0 }} />
      <FloatLine vertical sx={{ top: 0, right: 80 }} />
    </>
  );
}

function MetricStrip({ t }: { t: TFunction }) {
  return (
    <Grid container spacing={2} sx={{ mt: { xs: 6, md: 8 } }}>
      {METRICS.map((metric) => (
        <Grid key={metric.key} size={{ xs: 6, md: 3 }}>
          <Box component={m.div} variants={varFade('inUp', { distance: 24 })} sx={metricSx}>
            <Typography variant="h4">{metric.value}</Typography>
            <Typography variant="body2" sx={{ color: 'text.secondary' }}>
              {t(`home.operations.metrics.${metric.key}`)}
            </Typography>
          </Box>
        </Grid>
      ))}
    </Grid>
  );
}

type OperationCardProps = {
  t: TFunction;
  item: (typeof OPERATIONS)[number];
};

function OperationCard({ item, t }: OperationCardProps) {
  return (
    <Box component={m.div} variants={varFade('inUp', { distance: 24 })} sx={operationSx}>
      <Stack spacing={2.5}>
        <Box sx={iconSx}>
          <Iconify width={28} icon={item.icon} />
        </Box>
        <Typography variant="h6">{t(`home.operations.items.${item.key}.title`)}</Typography>
        <Typography variant="body2" sx={{ color: 'text.secondary' }}>
          {t(`home.operations.items.${item.key}.description`)}
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

const metricSx: SxProps<Theme> = [
  (theme) => ({
    p: 3,
    height: 1,
    borderRadius: 2,
    textAlign: 'center',
    bgcolor: varAlpha(theme.vars.palette.primary.mainChannel, 0.06),
    border: `solid 1px ${varAlpha(theme.vars.palette.primary.mainChannel, 0.12)}`,
  }),
];

const operationSx: SxProps<Theme> = [
  (theme) => ({
    p: 3,
    height: 1,
    borderRadius: 2,
    bgcolor: 'background.paper',
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.16)}`,
  }),
];

const iconSx: SxProps<Theme> = [
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
