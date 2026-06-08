'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import Box from '@mui/material/Box';

import { useSiteInfo } from 'src/actions/system-settings';

import { logoDisplaySource, isMaskableLogoSource } from './logo-utils';

type SiteLogoProps = {
  sx?: SxProps<Theme>;
  isSingle?: boolean;
};

export function SiteLogo({ isSingle = false, sx }: SiteLogoProps) {
  const site = useSiteInfo();

  if (site.error) {
    throw site.error;
  }

  const src = logoDisplaySource(site.data?.site_logo_base64 ?? '', { isSingle });

  if (!src) {
    return <Box aria-hidden sx={sx} />;
  }

  if (isMaskableLogoSource(src)) {
    return (
      <Box
        aria-hidden
        component="span"
        sx={[
          {
            width: 1,
            height: 1,
            color: 'text.primary',
            display: 'block',
            bgcolor: 'currentColor',
            mask: `url("${src}") center / contain no-repeat`,
            WebkitMask: `url("${src}") center / contain no-repeat`,
          },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
      />
    );
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
          display: 'block',
          objectFit: 'contain',
          objectPosition: 'center',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    />
  );
}
