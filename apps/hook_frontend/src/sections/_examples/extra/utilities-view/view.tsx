'use client';

import Box from '@mui/material/Box';

import { Styled } from './styled';
import { Countdown } from './countdown';
import { Gradients } from './gradients';
import { PhoneInputs } from './phone-inputs';
import { TextMaxLine } from './text-max-line';
import { ColorPickers } from './color-pickers';
import { NumberInputs } from './number-inputs';
import { CopyToClipboard } from './copy-to-clipboard';
import { contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Text max line',
    component: (
      <Box sx={contentStyles.grid()}>
        <TextMaxLine />
      </Box>
    ),
  },
  {
    name: 'Copy to clipboard',
    component: (
      <Box sx={contentStyles.grid()}>
        <CopyToClipboard />
      </Box>
    ),
  },
  {
    name: 'Gradient',
    component: <Gradients />,
  },
  {
    name: 'Countdown',
    component: (
      <Box sx={contentStyles.grid()}>
        <Countdown />
      </Box>
    ),
  },
  {
    name: 'Color pickers',
    component: (
      <Box sx={contentStyles.grid()}>
        <ColorPickers />
      </Box>
    ),
  },
  {
    name: 'Number input',
    component: (
      <Box sx={contentStyles.grid()}>
        <NumberInputs />
      </Box>
    ),
  },
  {
    name: 'Phone input',
    component: (
      <Box sx={contentStyles.grid()}>
        <PhoneInputs />
      </Box>
    ),
  },
  {
    name: 'Styled',
    component: <Styled />,
  },
];

// ----------------------------------------------------------------------

export function UtilitiesView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Utilities',
      }}
    />
  );
}
