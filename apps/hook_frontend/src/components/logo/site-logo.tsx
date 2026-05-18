'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import Box from '@mui/material/Box';

import { useSiteInfo } from 'src/actions/system-settings';

import { logoImageSource } from './logo-utils';

type SiteLogoProps = {
  sx?: SxProps<Theme>;
  isSingle?: boolean;
};

export function SiteLogo({ isSingle = true, sx }: SiteLogoProps) {
  const site = useSiteInfo();

  if (site.error) {
    throw site.error;
  }

  const src = logoImageSource(site.data?.site_logo_base64 ?? '');

  if (!src) {
    return <Box aria-hidden sx={sx} />;
  }

  return (
    <Box
      component="img"
      alt={site.data?.site_name ?? 'Logo'}
      src={src}
      sx={[
        {
          width: 1,
          height: 1,
          objectFit: isSingle ? 'contain' : 'contain',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    />
  );
}
