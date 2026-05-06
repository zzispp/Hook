'use client';

import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';

import { useCallback } from 'react';
import { upperFirst } from 'es-toolkit';
import { useCopyToClipboard } from 'minimal-shared/hooks';
import { parseCssVar, hexToRgbChannel } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import { colorKeys } from 'src/theme/core';

import { toast } from 'src/components/snackbar';

import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const GREY_SHADES = ['50', '100', '200', '300', '400', '500', '600', '700', '800', '900'] as const;
const SHADES = ['lighter', 'light', 'main', 'dark', 'darker'] as const;

const boxStyles: SxProps<Theme> = {
  display: 'grid',
  gridTemplateColumns: {
    xs: 'repeat(1, 1fr)',
    sm: 'repeat(2, 1fr)',
    md: 'repeat(3, 1fr)',
  },
};

// ----------------------------------------------------------------------

export function ColorsView() {
  const theme = useTheme();

  const { copy } = useCopyToClipboard();

  const handleCopy = useCallback(
    (color: string) => {
      if (color) {
        toast.success(`Copied: ${color}`);
        copy(color);
      }
    },
    [copy]
  );

  const DEMO_COMPONENTS = [
    ...colorKeys.palette.map((color) => ({
      name: upperFirst(color),
      component: (
        <Box sx={boxStyles}>
          {SHADES.map((shade) => (
            <ColorCard
              key={shade}
              shade={shade}
              hexColor={theme.palette[color][shade]}
              varColor={theme.vars.palette[color][shade]}
              onClick={() => handleCopy(theme.palette[color][shade])}
            />
          ))}
        </Box>
      ),
    })),
    {
      name: 'Grey',
      component: (
        <Box sx={boxStyles}>
          {GREY_SHADES.map((shade) => (
            <ColorCard
              key={shade}
              shade={shade}
              hexColor={theme.palette.grey[shade]}
              varColor={theme.vars.palette.grey[shade]}
              onClick={() => handleCopy(theme.palette.grey[shade])}
            />
          ))}
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Color',
        moreLinks: ['https://mui.com/customization/color', 'https://colors.eva.design'],
      }}
    />
  );
}

// ----------------------------------------------------------------------

type ColorCardProps = BoxProps & {
  shade: string;
  varColor: string;
  hexColor: string;
};

function ColorCard({ varColor, hexColor, shade, sx, ...other }: ColorCardProps) {
  return (
    <Box
      sx={[
        (theme) => ({
          px: 2,
          py: 3,
          display: 'flex',
          cursor: 'pointer',
          bgcolor: varColor,
          flexDirection: 'column',
          color: theme.palette.getContrastText(hexColor),
          transition: theme.transitions.create(['opacity'], {
            easing: theme.transitions.easing.sharp,
            duration: theme.transitions.duration.shorter,
          }),
          '&:hover': {
            opacity: 0.8,
          },
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Typography variant="subtitle2" sx={{ mb: 2, textTransform: 'capitalize' }}>
        {shade}
      </Typography>

      <Box
        sx={{
          gap: 1.5,
          display: 'flex',
          typography: 'caption',
          flexDirection: 'column',
          '& span': { opacity: 0.8, display: 'flex' },
          '& strong': { flexGrow: 1, fontWeight: 'fontWeightMedium' },
        }}
      >
        {[
          { label: 'Hex', value: hexColor },
          { label: 'Rgb', value: hexToRgbChannel(hexColor) },
          { label: 'Var', value: parseCssVar(varColor) },
        ].map((item) => (
          <span key={item.label}>
            <strong>{item.value}</strong> {item.label}
          </span>
        ))}
      </Box>
    </Box>
  );
}
