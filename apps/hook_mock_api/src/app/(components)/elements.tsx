'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import { alpha } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import colors from 'src/colors.json';

// ----------------------------------------------------------------------

type SectionProps = BoxProps & { title: React.ReactNode };

export function Section({ title, children, sx, ...other }: SectionProps) {
  return (
    <Box
      sx={[
        {
          p: 2,
          gap: 1.5,
          display: 'flex',
          borderRadius: 3,
          flexDirection: 'column',
          bgcolor: colors.grey[800],
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Typography variant="subtitle1" sx={{ fontWeight: 'fontWeightBold' }}>
        {title}
      </Typography>

      {children}
    </Box>
  );
}

// ----------------------------------------------------------------------

type HeadProps = {
  title: React.ReactNode;
  description: React.ReactNode;
};

export function TitleBlock({ title, description }: HeadProps) {
  return (
    <Box
      sx={{
        mb: 5,
        gap: 2,
        display: 'flex',
        textAlign: 'center',
        alignItems: 'center',
        flexDirection: 'column',
      }}
    >
      <Typography
        variant="h5"
        component="h1"
        sx={{
          backgroundClip: 'text',
          color: colors.primary.main,
          fontWeight: 'fontWeightBold',
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
          backgroundImage: `linear-gradient(45deg, ${colors.primary.main}, ${colors.warning.light})`,
        }}
      >
        {title}
      </Typography>
      <Typography
        typography="subtitle2"
        sx={{
          color: colors.grey[500],
          '& span': {
            color: colors.primary.light,
            fontWeight: 'fontWeightBold',
          },
        }}
      >
        {description}
      </Typography>
    </Box>
  );
}

// ----------------------------------------------------------------------

type BlockProps = BoxProps & {
  method: string;
  description?: string;
  path: React.ReactNode;
};

export function Block({ method, path, description, sx, ...other }: BlockProps) {
  const renderMethodLabel = () => (
    <Box
      component="span"
      sx={{
        py: 0.5,
        px: 0.75,
        lineHeight: 1,
        borderRadius: 1,
        color: 'common.white',
        fontWeight: 'fontWeightBold',
        fontSize: (theme) => theme.typography.pxToRem(11),
        ...(method === 'GET' && { bgcolor: colors.success.dark }),
        ...(method === 'POST' && { bgcolor: colors.info.dark }),
        ...(method === 'PUT' && { bgcolor: colors.warning.dark }),
        ...(method === 'PATCH' && { bgcolor: colors.secondary.dark }),
        ...(method === 'DELETE' && { bgcolor: colors.error.dark }),
      }}
    >
      {method}
    </Box>
  );

  return (
    <Box
      sx={[
        {
          p: 1.75,
          gap: 1,
          borderRadius: 2,
          display: 'flex',
          flexDirection: 'column',
          border: `dashed 1px ${alpha(colors.grey[500], 0.24)}`,
          '& strong': { color: colors.error.main },
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      {description && (
        <Typography variant="caption" sx={{ color: colors.grey[500] }}>
          {description}
        </Typography>
      )}

      <Box sx={{ gap: 1, display: 'flex', alignItems: 'center' }}>
        {renderMethodLabel()}
        <Box
          component="span"
          sx={{
            flexGrow: 1,
            fontSize: (theme) => theme.typography.pxToRem(14),
          }}
        >
          {path}
        </Box>
      </Box>
    </Box>
  );
}
