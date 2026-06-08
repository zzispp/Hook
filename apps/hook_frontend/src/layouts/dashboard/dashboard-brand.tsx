'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useSiteInfo } from 'src/actions/system-settings';

import { Logo } from 'src/components/logo';

type DashboardBrandProps = {
  sx?: SxProps<Theme>;
  compact?: boolean;
};

export function DashboardBrand({ compact = false, sx }: DashboardBrandProps) {
  const site = useSiteInfo();

  if (site.error) {
    throw site.error;
  }

  if (compact) {
    return <Logo isSingle sx={sx} />;
  }

  return (
    <Stack
      direction="row"
      spacing={1.25}
      sx={[{ minWidth: 0, alignItems: 'center' }, ...(Array.isArray(sx) ? sx : [sx])]}
    >
      <Logo isSingle />
      <Stack spacing={0.25} sx={{ minWidth: 0 }}>
        <Typography
          noWrap
          variant="subtitle2"
          sx={{ color: 'var(--layout-nav-text-primary-color)' }}
        >
          {site.data?.site_name}
        </Typography>
        <Typography
          noWrap
          variant="caption"
          sx={{ color: 'var(--layout-nav-text-disabled-color)' }}
        >
          {site.data?.site_subtitle}
        </Typography>
      </Stack>
    </Stack>
  );
}
