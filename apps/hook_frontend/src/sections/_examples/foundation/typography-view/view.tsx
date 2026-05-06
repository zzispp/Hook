'use client';

import type { Theme, SxProps, TypographyVariant } from '@mui/material/styles';

import { useMemo } from 'react';
import { remToPx } from 'minimal-shared/utils';
import { pickBy, upperFirst } from 'es-toolkit';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import { colorKeys } from 'src/theme/core';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const TEXT_COLORS = ['primary', 'secondary', 'disabled'] as const;
const VARIANTS: TypographyVariant[] = [
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'subtitle1',
  'subtitle2',
  'body1',
  'body2',
  'caption',
  'overline',
  'button',
];

const componentBoxStyles: SxProps<Theme> = {
  py: 3,
  flexDirection: 'column',
  alignItems: 'flex-start',
};

const LOREM_TEXT = `
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut
labore et dolore magna aliqua.
`;

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  ...VARIANTS.map((variant) => ({
    name: upperFirst(variant),
    component: <VariantPreview variant={variant} />,
  })),
  {
    name: 'Colors',
    component: (
      <Box
        sx={{
          gap: 3,
          display: 'grid',
          gridTemplateColumns: {
            xs: 'repeat(1, 1fr)',
            sm: 'repeat(2, 1fr)',
          },
        }}
      >
        <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
          {TEXT_COLORS.map((color) => (
            <ComponentBox
              key={color}
              sx={(theme) => ({
                ...componentBoxStyles,
                gap: 2,
                color: theme.vars.palette.text[color],
              })}
            >
              <Typography variant="subtitle1">Text {color}</Typography>
              <Typography variant="body2">{LOREM_TEXT}</Typography>
            </ComponentBox>
          ))}
        </Box>

        <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
          {colorKeys.palette.map((color) => (
            <ComponentBox
              key={color}
              sx={(theme) => ({
                ...componentBoxStyles,
                gap: 2,
                color: theme.vars.palette[color].main,
              })}
            >
              <Typography variant="subtitle1" sx={{ textTransform: 'capitalize' }}>
                {color}
              </Typography>
              <Typography variant="body2">{LOREM_TEXT}</Typography>
            </ComponentBox>
          ))}
        </Box>
      </Box>
    ),
  },
];

export function TypographyView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Typography',
        moreLinks: ['https://mui.com/components/typography'],
      }}
    />
  );
}

// ----------------------------------------------------------------------

type StyleValue = string | number | { [key: string]: StyleValue };

function formatStyleValues(value: StyleValue, key?: string): StyleValue {
  if (typeof value === 'string' && key === 'fontSize') {
    const pxValue = value.endsWith('rem') ? remToPx(value) : value;
    return `${value} (${pxValue}px)`;
  }

  if (typeof value === 'number' && key === 'lineHeight') {
    return Number(value.toFixed(2));
  }

  if (typeof value === 'object' && value !== null) {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, formatStyleValues(v, k)]));
  }

  return value;
}

type VariantPreviewProps = {
  variant: TypographyVariant;
};

function VariantPreview({ variant }: VariantPreviewProps) {
  const theme = useTheme();

  const filteredStyles = pickBy(
    theme.typography[variant],
    (_, key) =>
      ['fontSize', 'fontWeight', 'lineHeight', 'letterSpacing'].includes(key as string) ||
      (key as string).startsWith('@media')
  );

  const formattedStyles = useMemo(
    () => formatStyleValues(filteredStyles as StyleValue),
    [filteredStyles]
  );

  return (
    <ComponentBox
      sx={{
        flexWrap: 'unset',
        alignItems: 'flex-start',
        flexDirection: { xs: 'column', lg: 'row' },
      }}
    >
      <Typography variant={variant} sx={{ flexGrow: 1 }}>
        How can you choose a typeface?
      </Typography>

      <Box
        component="pre"
        sx={[
          {
            p: 2,
            m: 0,
            flexShrink: 0,
            borderRadius: 1.5,
            typography: 'body2',
            whiteSpace: 'pre-wrap',
            color: 'text.secondary',
            bgcolor: 'background.neutral',
            border: `dashed 1px ${theme.palette.divider}`,
            fontFamily: '"Lucida Console", Courier, monospace',
            width: { xs: 1, lg: 320 },
          },
        ]}
      >
        {JSON.stringify(formattedStyles, null, 2)}
      </Box>
    </ComponentBox>
  );
}
