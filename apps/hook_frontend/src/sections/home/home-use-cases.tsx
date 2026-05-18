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

// ----------------------------------------------------------------------

const USE_CASES = [
  {
    key: 'individual',
    icon: 'solar:user-id-bold',
  },
  {
    key: 'team',
    icon: 'solar:case-minimalistic-bold',
  },
  {
    key: 'admin',
    icon: 'solar:shield-check-bold',
  },
] as const;

type TFunction = ReturnType<typeof useTranslate>['t'];

export function HomeUseCases({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('common');

  return (
    <Box component="section" sx={[sectionSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <MotionViewport>
        <Container>
          <Grid container spacing={{ xs: 5, md: 8 }}>
            <Grid size={{ xs: 12, md: 4 }}>
              <SectionTitle
                caption={t('home.useCases.caption')}
                title={t('home.useCases.title')}
                txtGradient={t('home.useCases.gradient')}
                sx={{ textAlign: { xs: 'center', md: 'left' } }}
              />
            </Grid>

            <Grid size={{ xs: 12, md: 8 }}>
              <Grid container spacing={3}>
                {USE_CASES.map((item) => (
                  <Grid key={item.key} size={{ xs: 12, md: 4 }}>
                    <UseCaseCard item={item} t={t} />
                  </Grid>
                ))}
              </Grid>
            </Grid>
          </Grid>
        </Container>
      </MotionViewport>
    </Box>
  );
}

// ----------------------------------------------------------------------

type UseCaseCardProps = {
  t: TFunction;
  item: (typeof USE_CASES)[number];
};

function UseCaseCard({ item, t }: UseCaseCardProps) {
  return (
    <Stack component={m.div} variants={varFade('inUp', { distance: 24 })} spacing={3} sx={cardSx}>
      <Iconify width={36} icon={item.icon} />
      <Stack spacing={1.5}>
        <Typography variant="h5">{t(`home.useCases.items.${item.key}.title`)}</Typography>
        <Typography sx={{ color: 'text.secondary' }}>
          {t(`home.useCases.items.${item.key}.description`)}
        </Typography>
      </Stack>
    </Stack>
  );
}

// ----------------------------------------------------------------------

const sectionSx = {
  py: { xs: 10, md: 14 },
  bgcolor: 'background.neutral',
};

const cardSx: SxProps<Theme> = [
  (theme) => ({
    p: 3,
    height: 1,
    borderRadius: 2,
    color: 'text.primary',
    bgcolor: 'background.default',
    border: `solid 1px ${varAlpha(theme.vars.palette.grey['500Channel'], 0.16)}`,
  }),
];
