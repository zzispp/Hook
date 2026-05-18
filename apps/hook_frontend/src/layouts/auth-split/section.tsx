import type { BoxProps } from '@mui/material/Box';
import type { Breakpoint } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/global-config';
import { useTranslate } from 'src/locales/use-locales';

// ----------------------------------------------------------------------

export type AuthSplitSectionProps = BoxProps & {
  title?: string;
  imgUrl?: string;
  subtitle?: string;
  layoutQuery?: Breakpoint;
};

export function AuthSplitSection({
  sx,
  layoutQuery = 'md',
  title,
  imgUrl = `${CONFIG.assetsDir}/assets/illustrations/illustration-dashboard.webp`,
  subtitle,
  ...other
}: AuthSplitSectionProps) {
  const { t } = useTranslate('auth');
  const sectionTitle = title ?? t('layout.sectionTitle');
  const sectionSubtitle = subtitle ?? t('layout.sectionSubtitle');

  return (
    <Box
      sx={[
        (theme) => ({
          ...theme.mixins.bgGradient({
            images: [
              `linear-gradient(0deg, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.92)}, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.92)})`,
              `url(${CONFIG.assetsDir}/assets/background/background-3-blur.webp)`,
            ],
          }),
          px: 3,
          pb: 3,
          width: 1,
          maxWidth: 480,
          display: 'none',
          position: 'relative',
          pt: 'var(--layout-header-desktop-height)',
          [theme.breakpoints.up(layoutQuery)]: {
            gap: 8,
            display: 'flex',
            alignItems: 'center',
            flexDirection: 'column',
            justifyContent: 'center',
          },
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <div>
        <Typography variant="h3" sx={{ textAlign: 'center' }}>
          {sectionTitle}
        </Typography>

        {sectionSubtitle && (
          <Typography sx={{ color: 'text.secondary', textAlign: 'center', mt: 2 }}>
            {sectionSubtitle}
          </Typography>
        )}
      </div>

      <Box
        component="img"
        alt="Dashboard illustration"
        src={imgUrl}
        sx={{ width: 1, aspectRatio: '4/3', objectFit: 'cover' }}
      />
    </Box>
  );
}
