'use client';

import Box from '@mui/material/Box';

import { IconButtons } from './icon-buttons';
import { ButtonGroups } from './button-groups';
import { ToggleButtons } from './toggle-buttons';
import { ButtonVariant } from './button-variant';
import { contentStyles, ComponentLayout } from '../../layout';
import { FloatingActionButton } from './floating-action-button';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Contained',
    component: (
      <Box sx={contentStyles.column()}>
        <ButtonVariant variant="contained" />
      </Box>
    ),
  },
  {
    name: 'Outlined',
    component: (
      <Box sx={contentStyles.column()}>
        <ButtonVariant variant="outlined" />
      </Box>
    ),
  },
  {
    name: 'Text',
    component: (
      <Box sx={contentStyles.column()}>
        <ButtonVariant variant="text" />
      </Box>
    ),
  },
  {
    name: 'Soft',
    component: (
      <Box sx={contentStyles.column()}>
        <ButtonVariant variant="soft" />
      </Box>
    ),
  },
  {
    name: 'Icon button',
    component: (
      <Box sx={contentStyles.grid()}>
        <IconButtons />
      </Box>
    ),
  },
  {
    name: 'Fab button',
    component: (
      <Box sx={contentStyles.column()}>
        <FloatingActionButton />
      </Box>
    ),
  },
  {
    name: 'Button group',
    component: (
      <Box sx={contentStyles.grid()}>
        <ButtonGroups />
      </Box>
    ),
  },
  {
    name: 'Toggle button',
    component: (
      <Box sx={contentStyles.column()}>
        <ToggleButtons />
      </Box>
    ),
  },
];

// ----------------------------------------------------------------------

export function ButtonsView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Buttons',
        moreLinks: [
          'https://mui.com/material-ui/react-button/',
          'https://mui.com/material-ui/react-button-group/',
          'https://mui.com/material-ui/react-floating-action-button/',
          'https://mui.com/material-ui/react-toggle-button/',
        ],
      }}
    />
  );
}
