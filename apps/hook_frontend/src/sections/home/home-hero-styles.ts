import type { Transition, MotionProps } from 'framer-motion';
import type { Theme, SxProps, Breakpoint } from '@mui/material/styles';

import { varFade } from 'src/components/animate';

// ----------------------------------------------------------------------

export const smKey: Breakpoint = 'sm';
export const mdKey: Breakpoint = 'md';
export const lgKey: Breakpoint = 'lg';

export const motionProps: MotionProps = {
  variants: varFade('inUp', { distance: 24 }),
};

export const HERO_SIGNAL_KEYS = [
  'tokenManagement',
  'loadBalancing',
  'costQuota',
  'healthMonitoring',
] as const;

export const COMPATIBLE_TARGET_KEYS = ['openai', 'claude', 'gemini', 'cliTools'] as const;

export const gradientAnimation = { backgroundPosition: '200% center' };

export const gradientTransition: Transition = {
  duration: 20,
  ease: [0, 0, 1, 1],
  repeat: Infinity,
  repeatType: 'reverse' as const,
};

export const containerSx: SxProps<Theme> = [
  (theme) => ({
    py: 3,
    gap: 5,
    zIndex: 9,
    display: 'flex',
    alignItems: 'center',
    flexDirection: 'column',
    [theme.breakpoints.up(mdKey)]: {
      flex: '1 1 auto',
      justifyContent: 'center',
      py: 'var(--layout-header-desktop-height)',
    },
  }),
];

export const headingSx: SxProps<Theme> = [
  (theme) => ({
    my: 0,
    mx: 'auto',
    maxWidth: 760,
    display: 'flex',
    flexWrap: 'wrap',
    typography: 'h2',
    justifyContent: 'center',
    fontFamily: theme.typography.fontSecondaryFamily,
    [theme.breakpoints.up(lgKey)]: {
      fontSize: theme.typography.pxToRem(72),
      lineHeight: '90px',
    },
  }),
];

export const brandSx: SxProps<Theme> = [
  (theme) => ({
    ...theme.mixins.textGradient(
      `300deg, ${theme.vars.palette.primary.main} 0%, ${theme.vars.palette.warning.main} 25%, ${theme.vars.palette.primary.main} 50%, ${theme.vars.palette.warning.main} 75%, ${theme.vars.palette.primary.main} 100%`
    ),
    backgroundSize: '400%',
    mx: { xs: 0.75, md: 1, xl: 1.5 },
  }),
];

export const textSx: SxProps<Theme> = [
  (theme) => ({
    mx: 'auto',
    maxWidth: 720,
    color: 'text.secondary',
    [theme.breakpoints.up(smKey)]: { whiteSpace: 'pre-line' },
    [theme.breakpoints.up(lgKey)]: { fontSize: 20, lineHeight: '36px' },
  }),
];

export const signalsSx = {
  gap: 1.5,
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  typography: 'subtitle2',
  justifyContent: 'center',
};

export const signalItemSx = { gap: 0.75, display: 'inline-flex', alignItems: 'center' };

export const buttonsSx = {
  gap: { xs: 1.5, sm: 2 },
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'center',
};

export const compatibilitySx = {
  gap: 1,
  display: 'flex',
  flexWrap: 'wrap',
  justifyContent: 'center',
};
