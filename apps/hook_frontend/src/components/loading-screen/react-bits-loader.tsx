'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';

// ----------------------------------------------------------------------


const LOADER_BACKGROUND = '#120F17';
const LIGHT_BACKGROUND = '#FFFFFF';
const LIGHT_LOGO_COLOR = '#1C252E';
const DARK_LOGO_COLOR = '#FFFFFF';
const LOADER_TRANSITION =
  'opacity 0.6s cubic-bezier(0.4, 0, 0.2, 1), visibility 0.6s cubic-bezier(0.4, 0, 0.2, 1)';

export type ReactBitsLoaderMode = 'auto' | 'dark' | 'light';

export type ReactBitsLoaderProps = BoxProps & {
  hiding?: boolean;
  mode?: ReactBitsLoaderMode;
};

export function ReactBitsLoader({
  className,
  hiding = false,
  mode = 'auto',
  sx,
  ...other
}: ReactBitsLoaderProps) {
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
          color: 'var(--react-bits-loader-fg)',
          background: 'var(--react-bits-loader-bg)',
          opacity: hiding ? 0 : 1,
          visibility: hiding ? 'hidden' : 'visible',
          pointerEvents: hiding ? 'none' : 'auto',
          transition: LOADER_TRANSITION,
          '@keyframes react-bits-loader-pulse': {
            '0%, 100%': { opacity: 0.3, transform: 'scale(1)' },
            '50%': { opacity: 0.7, transform: 'scale(1.08)' },
          },
          '& .ln-loader-logo': {
            opacity: 0.6,
            animation: 'react-bits-loader-pulse 1.8s ease-in-out infinite',
          },
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <ReactBitsLoaderLogo />
    </Box>
  );
}

function ReactBitsLoaderLogo() {
  return (
    <svg
      className="ln-loader-logo"
      width="40"
      height="40"
      viewBox="0 0 36 36"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <g stroke="currentColor" strokeWidth="5" strokeLinecap="round" strokeLinejoin="round">
        <path d="M 30 4 V 20 A 12 12 0 0 1 6 20 A 6 6 0 0 1 12 14" />
      </g>
      <circle cx="21" cy="11" r="3.5" fill="currentColor" opacity="0.6" />
    </svg>
  );
}

function getModeStyles(mode: ReactBitsLoaderMode) {
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
    '--react-bits-loader-bg': LIGHT_BACKGROUND,
    '--react-bits-loader-fg': LIGHT_LOGO_COLOR,
  };
}

function darkModeVars() {
  return {
    '--react-bits-loader-bg': LOADER_BACKGROUND,
    '--react-bits-loader-fg': DARK_LOGO_COLOR,
  };
}
