'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';

import { LogoIcon } from 'src/home/components/common/SVGComponents';

// ----------------------------------------------------------------------


const LOADER_BACKGROUND = '#120F17';
const LIGHT_BACKGROUND = '#FFFFFF';
const LIGHT_LOGO_COLOR = '#1C252E';
const DARK_LOGO_COLOR = '#FFFFFF';
const LOADER_TRANSITION =
  'opacity 0.6s cubic-bezier(0.4, 0, 0.2, 1), visibility 0.6s cubic-bezier(0.4, 0, 0.2, 1)';

export type HomeLoaderMode = 'auto' | 'dark' | 'light';

export type HomeLoaderProps = BoxProps & {
  hiding?: boolean;
  mode?: HomeLoaderMode;
};

export function HomeLoader({
  className,
  hiding = false,
  mode = 'auto',
  sx,
  ...other
}: HomeLoaderProps) {
  const classes = ['ln-loader', hiding ? 'ln-loader--hide' : '', className]
    .filter(Boolean)
    .join(' ');

  return (
    <Box
      className={classes}
      sx={[
        getModeStyles(mode),
        {
          inset: 0,
          zIndex: 9999,
          display: 'flex',
          position: 'fixed',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'var(--home-loader-fg)',
          background: 'var(--home-loader-bg)',
          opacity: hiding ? 0 : 1,
          visibility: hiding ? 'hidden' : 'visible',
          pointerEvents: hiding ? 'none' : 'auto',
          transition: LOADER_TRANSITION,
          '@keyframes home-loader-pulse': {
            '0%, 100%': { opacity: 0.3, transform: 'scale(1)' },
            '50%': { opacity: 0.7, transform: 'scale(1.08)' },
          },
          '& .ln-loader-logo': {
            width: 40,
            height: 40,
            opacity: 0.6,
            color: 'var(--home-loader-fg)',
            animation: 'home-loader-pulse 1.8s ease-in-out infinite',
          },
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <LogoIcon className="ln-loader-logo" aria-hidden />
    </Box>
  );
}

function getModeStyles(mode: HomeLoaderMode) {
  if (mode === 'dark') {
    return darkModeVars();
  }

  if (mode === 'light') {
    return lightModeVars();
  }

  return {
    ...lightModeVars(),
    '[data-color-scheme="dark"] &': darkModeVars(),
  };
}

function lightModeVars() {
  return {
    '--home-loader-bg': LIGHT_BACKGROUND,
    '--home-loader-fg': LIGHT_LOGO_COLOR,
  };
}

function darkModeVars() {
  return {
    '--home-loader-bg': LOADER_BACKGROUND,
    '--home-loader-fg': DARK_LOGO_COLOR,
  };
}
