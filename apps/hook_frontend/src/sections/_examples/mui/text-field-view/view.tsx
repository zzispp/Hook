'use client';

import Box from '@mui/material/Box';

import { TextFieldVariant } from './text-field-variant';
import { contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Outlined',
    component: (
      <Box sx={contentStyles.grid()}>
        <TextFieldVariant variant="outlined" />
      </Box>
    ),
  },
  {
    name: 'Filled',
    component: (
      <Box sx={contentStyles.grid()}>
        <TextFieldVariant variant="filled" />
      </Box>
    ),
  },
  {
    name: 'Standard',
    component: (
      <Box sx={contentStyles.grid()}>
        <TextFieldVariant variant="standard" />
      </Box>
    ),
  },
];

// ----------------------------------------------------------------------

export function TextFieldView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Text field',
        moreLinks: ['https://mui.com/material-ui/react-text-field/'],
      }}
    />
  );
}
