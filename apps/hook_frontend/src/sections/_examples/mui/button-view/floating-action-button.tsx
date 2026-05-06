import type { FabProps } from '@mui/material/Fab';

import { m } from 'framer-motion';
import { upperFirst } from 'es-toolkit';

import Fab from '@mui/material/Fab';
import Divider from '@mui/material/Divider';
import Tooltip from '@mui/material/Tooltip';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';
import { varTap, transitionTap } from 'src/components/animate';

import { ComponentBox } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium', 'large'] as const;
const COLORS = ['default', 'inherit', ...colorKeys.palette] as const;

// ----------------------------------------------------------------------

export function FloatingActionButton() {
  const renderIcon = () => <Iconify icon="solar:letter-outline" width={24} />;

  const renderDivider = () => <Divider sx={{ width: 1, borderStyle: 'dashed' }} />;

  const renderVariant = (title: string, variant: FabProps['variant'][]) => {
    const circularVariant = variant[0];
    const extendedVariant = variant[1];

    return (
      <ComponentBox title={title}>
        {COLORS.map((color) => (
          <Tooltip key={color} title={color}>
            <Fab color={color} variant={circularVariant}>
              {renderIcon()}
            </Fab>
          </Tooltip>
        ))}

        {renderDivider()}

        {COLORS.map((color) => (
          <Fab key={color} color={color} variant={extendedVariant}>
            {renderIcon()}
            {upperFirst(color)}
          </Fab>
        ))}

        {renderDivider()}

        {SIZES.map((size) => (
          <Tooltip key={size} title={size}>
            <Fab size={size} color="info" variant={circularVariant}>
              {renderIcon()}
            </Fab>
          </Tooltip>
        ))}

        {SIZES.map((size) => (
          <Fab key={size} size={size} color="info" variant={extendedVariant}>
            {renderIcon()}
            {upperFirst(size)}
          </Fab>
        ))}

        {renderDivider()}

        <Fab color="info" disabled variant={circularVariant}>
          {renderIcon()}
        </Fab>
        <Fab color="info" disabled variant={extendedVariant}>
          {renderIcon()}
          Disabled
        </Fab>
      </ComponentBox>
    );
  };

  return (
    <>
      {renderVariant('Default', ['circular', 'extended'])}
      {renderVariant('Outlined', ['outlined', 'outlinedExtended'])}
      {renderVariant('Soft', ['soft', 'softExtended'])}

      <ComponentBox title="With Animate">
        {SIZES.map((size) => (
          <Fab
            key={size}
            component={m.button}
            whileTap={varTap()}
            transition={transitionTap()}
            variant="softExtended"
            color="info"
            size={size}
          >
            <Iconify icon="solar:letter-outline" width={24} />
            {upperFirst(size)}
          </Fab>
        ))}
      </ComponentBox>
    </>
  );
}
