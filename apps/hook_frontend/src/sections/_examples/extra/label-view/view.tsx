'use client';

import type { LabelProps } from 'src/components/label';

import Tooltip from '@mui/material/Tooltip';

import { colorKeys } from 'src/theme/core';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const VARIANTS = ['filled', 'outlined', 'soft', 'inverted'] as const;
const COLORS = ['default', ...colorKeys.palette, ...colorKeys.common] as const;

// ----------------------------------------------------------------------

const renderIcon = () => <Iconify icon="solar:letter-bold" />;

const DEMO_COMPONENTS = [
  {
    name: 'Filled',
    component: (
      <ComponentBox>
        {COLORS.map((color) => (
          <Tooltip key={color} title={color}>
            <Label color={color} variant="filled">
              {color}
            </Label>
          </Tooltip>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Outlined',
    component: (
      <ComponentBox>
        {COLORS.map((color) => (
          <Label key={color} color={color} variant="outlined">
            {color}
          </Label>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Soft',
    component: (
      <ComponentBox>
        {COLORS.map((color) => (
          <Label key={color} color={color} variant="soft">
            {color}
          </Label>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Inverted',
    component: (
      <ComponentBox>
        {COLORS.map((color) => (
          <Label key={color} color={color} variant="inverted">
            {color}
          </Label>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'With icon',
    component: (
      <ComponentBox>
        {VARIANTS.map((variant, index) => {
          const isStartIcon = [0, 1].includes(index);

          const labelProps: Partial<LabelProps> = isStartIcon
            ? { color: 'primary', startIcon: renderIcon() }
            : { color: 'secondary', endIcon: renderIcon() };

          return (
            <Label {...labelProps} key={variant} variant={variant}>
              {isStartIcon ? 'Start icon' : 'End icon'}
            </Label>
          );
        })}
      </ComponentBox>
    ),
  },
];

// ----------------------------------------------------------------------

export function LabelView() {
  return <ComponentLayout sectionData={DEMO_COMPONENTS} heroProps={{ heading: 'Label' }} />;
}
