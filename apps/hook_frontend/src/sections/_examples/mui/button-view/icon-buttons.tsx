import { m } from 'framer-motion';

import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';
import { varTap, varHover, transitionTap } from 'src/components/animate';

import { ComponentBox } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium', 'large'] as const;
const COLORS = ['inherit', 'default', ...colorKeys.palette] as const;

// ----------------------------------------------------------------------

export function IconButtons() {
  const renderIcon = (size: (typeof SIZES)[number] = 'medium') => (
    <Iconify
      icon="solar:letter-outline"
      width={(size === 'small' && 18) || (size === 'medium' && 20) || 24}
    />
  );

  return (
    <>
      <ComponentBox title="Base">
        <IconButton color="inherit">{renderIcon()}</IconButton>
        <IconButton>{renderIcon()}</IconButton>
        <IconButton color="primary">{renderIcon()}</IconButton>
        <IconButton color="secondary">{renderIcon()}</IconButton>
        <IconButton disabled>{renderIcon()}</IconButton>
      </ComponentBox>

      <ComponentBox title="Colors">
        {COLORS.map((color) => (
          <Tooltip key={color} title={color}>
            <IconButton color={color}>{renderIcon()}</IconButton>
          </Tooltip>
        ))}
      </ComponentBox>

      <ComponentBox title="Sizes">
        {SIZES.map((size) => (
          <Tooltip key={size} title={size}>
            <IconButton size={size} color="info">
              {renderIcon(size)}
            </IconButton>
          </Tooltip>
        ))}
      </ComponentBox>

      <ComponentBox title="With animate">
        {SIZES.map((size) => (
          <IconButton
            key={size}
            component={m.button}
            whileTap={varTap()}
            whileHover={varHover(1.05)}
            transition={transitionTap()}
            size={size}
            color="error"
          >
            {renderIcon(size)}
          </IconButton>
        ))}
      </ComponentBox>
    </>
  );
}
