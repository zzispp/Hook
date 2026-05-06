'use client';

import type { PaperProps } from '@mui/material/Paper';
import type { Theme, SxProps } from '@mui/material/styles';
import type { CustomShadows, PaletteColorKey } from 'src/theme/core';

import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import { useTheme } from '@mui/material/styles';

import { colorKeys } from 'src/theme/core';

import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const boxStyles: SxProps<Theme> = {
  gap: 3,
  display: 'grid',
  gridTemplateColumns: {
    xs: 'repeat(2, 1fr)',
    md: 'repeat(4, 1fr)',
  },
};

// ----------------------------------------------------------------------

export function ShadowsView() {
  const theme = useTheme();

  const SHADOWS = theme.vars.shadows.slice(1);

  const CUSTOM_SHADOWS = Object.keys(theme.vars.customShadows)
    .filter((key) => !colorKeys.palette.includes(key as PaletteColorKey))
    .map((key) => key) as (keyof CustomShadows)[];

  const DEMO_COMPONENTS = [
    {
      name: 'System',
      description: 'Default shadows of Mui.',
      component: (
        <Box sx={boxStyles}>
          {SHADOWS.map((elevation, index) => (
            <ShadowCard key={elevation} title={`z${index + 1}`} sx={{ boxShadow: elevation }} />
          ))}
        </Box>
      ),
    },
    {
      name: 'Custom',
      description: 'Extended shadows ​​are used in this template.',
      component: (
        <Box sx={boxStyles}>
          {CUSTOM_SHADOWS.map((elevation) => (
            <ShadowCard
              key={elevation}
              title={elevation}
              sx={{
                boxShadow: theme.customShadows[elevation],
                ...(!elevation.startsWith('z') && {
                  textTransform: 'capitalize',
                }),
              }}
            />
          ))}
        </Box>
      ),
    },
    {
      name: 'Colors',
      description: 'Extended shadows ​​are used in this template.',
      component: (
        <Box
          sx={{
            ...boxStyles,
            gridTemplateColumns: {
              xs: 'repeat(1, 1fr)',
              md: 'repeat(2, 1fr)',
            },
          }}
        >
          {colorKeys.palette.map((color) => (
            <Box
              key={color}
              sx={{
                gap: 1,
                display: 'grid',
                gridTemplateColumns: {
                  xs: 'repeat(1, 1fr)',
                  sm: 'repeat(2, 1fr)',
                },
              }}
            >
              <ShadowCard
                title={color}
                sx={{
                  textTransform: 'capitalize',
                  color: theme.vars.palette[color].contrastText,
                  bgcolor: theme.vars.palette[color].main,
                  boxShadow: theme.vars.customShadows[color],
                }}
              />

              <ShadowCard
                title={color}
                sx={{
                  textTransform: 'capitalize',
                  color: theme.vars.palette[color].dark,
                  boxShadow: theme.vars.customShadows[color],
                }}
              />
            </Box>
          ))}
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Shadows',
      }}
    />
  );
}

// ----------------------------------------------------------------------

type ShadowCardProps = PaperProps & {
  title: string;
};

function ShadowCard({ sx, title, ...other }: ShadowCardProps) {
  return (
    <Paper
      sx={[
        {
          p: 3,
          minHeight: 120,
          display: 'flex',
          alignItems: 'center',
          typography: 'subtitle2',
          justifyContent: 'center',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      {title}
    </Paper>
  );
}
