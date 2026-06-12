'use client';

import type { HomeLoaderProps } from './home-loader';

import { Fragment } from 'react';

import Portal from '@mui/material/Portal';

import { HomeLoader } from './home-loader';

// ----------------------------------------------------------------------

export type SplashScreenProps = HomeLoaderProps & {
  portal?: boolean;
};

export function SplashScreen({ portal = true, ...other }: SplashScreenProps) {
  const PortalWrapper = portal ? Portal : Fragment;

  return (
    <PortalWrapper>
      <HomeLoader {...other} />
    </PortalWrapper>
  );
}
