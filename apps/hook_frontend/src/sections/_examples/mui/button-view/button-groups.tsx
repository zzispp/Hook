import type { ButtonGroupProps } from '@mui/material/ButtonGroup';

import Button from '@mui/material/Button';
import ButtonGroup from '@mui/material/ButtonGroup';

import { colorKeys } from 'src/theme/core';

import { ComponentBox } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium', 'large'] as const;
const VARIANTS = ['contained', 'outlined', 'text', 'soft'] as const;
const COLORS = ['inherit', ...colorKeys.palette, ...colorKeys.common] as const;

// ----------------------------------------------------------------------

export function ButtonGroups() {
  const renderContent = () => [
    <Button key="one">One</Button>,
    <Button key="two">Two</Button>,
    <Button key="three">Three</Button>,
    <Button key="four">Four</Button>,
  ];

  const renderVariant = (title: string, variant: ButtonGroupProps['variant']) => (
    <ComponentBox title={title}>
      {COLORS.map((color) => (
        <ButtonGroup key={color} variant={variant} color={color}>
          {renderContent()}
        </ButtonGroup>
      ))}
      <ButtonGroup disabled variant={variant} color="info">
        {renderContent()}
      </ButtonGroup>
    </ComponentBox>
  );

  return (
    <>
      {renderVariant('Contained', 'contained')}
      {renderVariant('Outlined', 'outlined')}
      {renderVariant('Text', 'text')}
      {renderVariant('Soft', 'soft')}

      <ComponentBox title="Sizes" sx={{ flexDirection: 'column', justifyContent: 'flex-start' }}>
        {SIZES.map((size) => (
          <ButtonGroup key={size} size={size} variant="contained">
            {renderContent()}
          </ButtonGroup>
        ))}
      </ComponentBox>

      <ComponentBox title="Orientation">
        {VARIANTS.map((variant) => (
          <ButtonGroup key={variant} variant={variant} orientation="vertical">
            {renderContent()}
          </ButtonGroup>
        ))}
        <ButtonGroup disabled variant="soft" color="info" orientation="vertical">
          {renderContent()}
        </ButtonGroup>
      </ComponentBox>
    </>
  );
}
