import { upperFirst } from 'es-toolkit';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, contentStyles } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium', 'large', 'xLarge'] as const;
const COLORS = ['inherit', ...colorKeys.palette, ...colorKeys.common] as const;

// ----------------------------------------------------------------------

type Props = {
  variant: 'text' | 'contained' | 'outlined' | 'soft';
};

export function ButtonVariant({ variant }: Props) {
  const renderIcon = () => <Iconify icon="solar:letter-outline" />;

  return (
    <>
      <ComponentBox title="Base" sx={contentStyles.row()}>
        <Button variant={variant} color="inherit">
          Inherit
        </Button>
        <Button variant={variant} color="primary">
          Primary
        </Button>
        <Button variant={variant} color="secondary">
          Secondary
        </Button>
        <Button variant={variant} color="white" disabled>
          Disabled
        </Button>
        <Button variant={variant}>Link</Button>
      </ComponentBox>

      <ComponentBox title="Colors" sx={contentStyles.row()}>
        {COLORS.map((color) => (
          <Button key={color} variant={variant} color={color}>
            {upperFirst(color)}
          </Button>
        ))}
      </ComponentBox>

      <ComponentBox title="With icon & loading" sx={contentStyles.row()}>
        <Button color="error" variant={variant} startIcon={renderIcon()}>
          Icon left
        </Button>
        <Button variant={variant} color="error" endIcon={renderIcon()}>
          Icon right
        </Button>
        <Button loading variant={variant}>
          Submit
        </Button>
        <Button loading loadingIndicator="Loading..." variant={variant}>
          Fetch data
        </Button>
        <Button
          loading
          size="large"
          loadingPosition="start"
          startIcon={renderIcon()}
          variant={variant}
        >
          Start
        </Button>
        <Button loading size="large" loadingPosition="end" endIcon={renderIcon()} variant={variant}>
          End
        </Button>
      </ComponentBox>

      <ComponentBox title="Sizes">
        <Box sx={contentStyles.row()}>
          {SIZES.map((size) => (
            <Button key={size} variant={variant} color="info" size={size}>
              {upperFirst(size)}
            </Button>
          ))}
        </Box>

        <Box sx={contentStyles.row()}>
          {SIZES.map((size) => (
            <Button
              key={size}
              loading
              size={size}
              loadingPosition="start"
              startIcon={renderIcon()}
              variant={variant}
            >
              {upperFirst(size)}
            </Button>
          ))}
        </Box>

        <Box sx={contentStyles.row()}>
          {SIZES.map((size) => (
            <Button
              key={size}
              loading
              size={size}
              loadingPosition="end"
              endIcon={renderIcon()}
              variant={variant}
            >
              {upperFirst(size)}
            </Button>
          ))}
        </Box>
      </ComponentBox>
    </>
  );
}
