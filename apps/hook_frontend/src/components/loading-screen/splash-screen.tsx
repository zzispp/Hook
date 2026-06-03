'use client';

import type { ReactBitsLoaderProps } from './react-bits-loader';

import { Fragment } from 'react';

import Portal from '@mui/material/Portal';

import { ReactBitsLoader } from './react-bits-loader';

// ----------------------------------------------------------------------

export type SplashScreenProps = ReactBitsLoaderProps & {
  portal?: boolean;
};

export function SplashScreen({ portal = true, ...other }: SplashScreenProps) {
  const PortalWrapper = portal ? Portal : Fragment;

  return (
    <PortalWrapper>
      <ReactBitsLoader {...other} />
    </PortalWrapper>
  );
}
