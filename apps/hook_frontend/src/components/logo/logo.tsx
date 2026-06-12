'use client';

import type { LinkProps } from '@mui/material/Link';

import { mergeClasses } from 'minimal-shared/utils';

import Link from '@mui/material/Link';
import { styled } from '@mui/material/styles';

import { RouterLink } from 'src/routes/components';

import { SiteLogo } from './site-logo';
import { logoClasses } from './classes';

// ----------------------------------------------------------------------

const LOGO_COMPACT_SIZE = 40;
const LOGO_FULL_WIDTH = 102;
const LOGO_FULL_HEIGHT = 36;

export type LogoProps = LinkProps & {
  isSingle?: boolean;
  disabled?: boolean;
  label?: string;
  source?: string;
};

export function Logo({
  sx,
  label,
  source,
  disabled,
  className,
  href = '/',
  isSingle = false,
  ...other
}: LogoProps) {
  const logoDimensions = isSingle
    ? { width: LOGO_COMPACT_SIZE, height: LOGO_COMPACT_SIZE }
    : { width: LOGO_FULL_WIDTH, height: LOGO_FULL_HEIGHT };

  return (
    <LogoRoot
      component={RouterLink}
      href={href}
      aria-label="Logo"
      underline="none"
      className={mergeClasses([logoClasses.root, className])}
      sx={[
        {
          ...logoDimensions,
          ...(disabled && { pointerEvents: 'none' }),
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <SiteLogo isSingle={isSingle} label={label} source={source} />
    </LogoRoot>
  );
}

// ----------------------------------------------------------------------

const LogoRoot = styled(Link)(() => ({
  flexShrink: 0,
  color: 'transparent',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  lineHeight: 0,
  verticalAlign: 'middle',
}));
