import type { BoxProps } from '@mui/material/Box';
import type { MotionValue, SpringOptions } from 'framer-motion';

import { useRef, useState } from 'react';
import { m, useScroll, useSpring, useTransform, useMotionValueEvent } from 'framer-motion';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import useMediaQuery from '@mui/material/useMediaQuery';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { useTranslate } from 'src/locales/use-locales';
import { useSiteInfo } from 'src/actions/system-settings';

import { Iconify } from 'src/components/iconify';
import { varFade, MotionContainer } from 'src/components/animate';

import { HeroBackground } from './components/hero-background';
import {
  mdKey,
  textSx,
  brandSx,
  buttonsSx,
  headingSx,
  signalsSx,
  motionProps,
  containerSx,
  signalItemSx,
  compatibilitySx,
  HERO_SIGNAL_KEYS,
  gradientAnimation,
  gradientTransition,
  COMPATIBLE_TARGET_KEYS,
} from './home-hero-styles';

// ----------------------------------------------------------------------

export function HomeHero({ sx, ...other }: BoxProps) {
  const scrollProgress = useScrollPercent();
  const mdUp = useMediaQuery((theme) => theme.breakpoints.up(mdKey));
  const distance = mdUp ? scrollProgress.percent : 0;

  const y1 = useTransformY(scrollProgress.scrollY, distance * -7);
  const y2 = useTransformY(scrollProgress.scrollY, distance * -6);
  const y3 = useTransformY(scrollProgress.scrollY, distance * -5);
  const y4 = useTransformY(scrollProgress.scrollY, distance * -4);
  const y5 = useTransformY(scrollProgress.scrollY, distance * -3);

  const opacity: MotionValue<number> = useTransform(
    scrollProgress.scrollY,
    [0, 1],
    [1, mdUp ? Number((1 - scrollProgress.percent / 100).toFixed(1)) : 1]
  );

  return (
    <Box
      ref={scrollProgress.elementRef}
      component="section"
      sx={[
        (theme) => ({
          overflow: 'hidden',
          position: 'relative',
          [theme.breakpoints.up(mdKey)]: {
            minHeight: 760,
            height: '100vh',
            maxHeight: 1440,
            display: 'block',
            willChange: 'opacity',
            mt: 'calc(var(--layout-header-desktop-height) * -1)',
          },
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Box
        component={m.div}
        style={{ opacity }}
        sx={[
          (theme) => ({
            width: 1,
            display: 'flex',
            position: 'relative',
            flexDirection: 'column',
            transition: theme.transitions.create(['opacity']),
            [theme.breakpoints.up(mdKey)]: {
              height: 1,
              position: 'fixed',
              maxHeight: 'inherit',
            },
          }),
        ]}
      >
        <Container component={MotionContainer} sx={containerSx}>
          <Stack spacing={3} sx={{ textAlign: 'center' }}>
            <m.div style={{ y: y1 }}>
              <HeroHeading />
            </m.div>
            <m.div style={{ y: y2 }}>
              <HeroText />
            </m.div>
          </Stack>

          <m.div style={{ y: y3 }}>
            <HeroSignals />
          </m.div>
          <m.div style={{ y: y4 }}>
            <HeroButtons />
          </m.div>
          <m.div style={{ y: y5 }}>
            <HeroCompatibility />
          </m.div>
        </Container>

        <HeroBackground />
      </Box>
    </Box>
  );
}

// ----------------------------------------------------------------------

function HeroHeading() {
  const site = useSiteInfo();

  if (site.error) {
    throw site.error;
  }

  return (
    <m.div {...motionProps}>
      <Box component="h1" sx={headingSx}>
        <Box component={m.span} animate={gradientAnimation} transition={gradientTransition} sx={brandSx}>
          {site.data?.site_name}
        </Box>
        {site.data?.site_subtitle}
      </Box>
    </m.div>
  );
}

function HeroText() {
  const { t } = useTranslate('common');

  return (
    <m.div {...motionProps}>
      <Typography variant="body2" sx={textSx}>{t('home.hero.description')}</Typography>
    </m.div>
  );
}

function HeroSignals() {
  const { t } = useTranslate('common');

  return (
    <Box sx={signalsSx}>
      {HERO_SIGNAL_KEYS.map((signal) => (
        <Box key={signal} component={m.div} variants={varFade('in')} sx={signalItemSx}>
          <Iconify width={16} icon="eva:checkmark-fill" />
          {t(`home.hero.signals.${signal}`)}
        </Box>
      ))}
    </Box>
  );
}

function HeroButtons() {
  const { t } = useTranslate('common');

  return (
    <Box sx={buttonsSx}>
      <m.div {...motionProps}>
        <Button
          component={RouterLink}
          href={paths.dashboard.root}
          color="inherit"
          size="large"
          variant="contained"
          startIcon={<Iconify width={24} icon="custom:flash-outline" />}
          sx={{ height: 52 }}
        >
          {t('home.hero.primaryAction')}
        </Button>
      </m.div>

      <m.div {...motionProps}>
        <Button
          component={RouterLink}
          href={paths.dashboard.models}
          color="inherit"
          size="large"
          variant="outlined"
          startIcon={<Iconify width={22} icon="solar:box-minimalistic-bold" />}
          sx={{ height: 52, borderColor: 'currentColor' }}
        >
          {t('home.hero.secondaryAction')}
        </Button>
      </m.div>
    </Box>
  );
}

function HeroCompatibility() {
  const { t } = useTranslate('common');

  return (
    <Stack spacing={3} sx={{ textAlign: 'center', alignItems: 'center' }}>
      <m.div {...motionProps}>
        <Typography variant="overline" sx={{ opacity: 0.4 }}>
          {t('home.hero.compatibilityCaption')}
        </Typography>
      </m.div>

      <Box sx={compatibilitySx}>
        {COMPATIBLE_TARGET_KEYS.map((target) => (
          <Chip key={target} label={t(`home.hero.compatibleTargets.${target}`)} variant="outlined" size="small" />
        ))}
      </Box>
    </Stack>
  );
}

// ----------------------------------------------------------------------

function useTransformY(value: MotionValue<number>, distance: number) {
  const physics: SpringOptions = {
    mass: 0.1,
    damping: 20,
    stiffness: 300,
    restDelta: 0.001,
  };

  return useSpring(useTransform(value, [0, 1], [0, distance]), physics);
}

function useScrollPercent() {
  const elementRef = useRef<HTMLDivElement>(null);
  const { scrollY } = useScroll();
  const [percent, setPercent] = useState(0);

  useMotionValueEvent(scrollY, 'change', (scrollHeight) => {
    const heroHeight = elementRef.current?.offsetHeight ?? 1;
    const scrollPercent = Math.floor((scrollHeight / heroHeight) * 100);

    setPercent(scrollPercent >= 100 ? 100 : scrollPercent);
  });

  return { elementRef, percent, scrollY };
}
