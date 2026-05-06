'use client';

import Box from '@mui/material/Box';

import { paths } from 'src/routes/paths';

import { PickerTime } from './picker-time';
import { PickerDate } from './picker-date';
import { PickerDateTime } from './picker-date-time';
import { PickerDateRange } from './picker-date-range';
import { contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Date',
    component: (
      <Box sx={contentStyles.column()}>
        <PickerDate />
      </Box>
    ),
  },
  {
    name: 'Time',
    component: (
      <Box sx={contentStyles.column()}>
        <PickerTime />
      </Box>
    ),
  },
  {
    name: 'Date & Time',
    component: (
      <Box sx={contentStyles.column()}>
        <PickerDateTime />
      </Box>
    ),
  },
  { name: 'Range', component: <PickerDateRange /> },
];

// ----------------------------------------------------------------------

export function DatePickersView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'MUI X Date and Time Pickers',
        links: [
          { name: 'Components', href: paths.components },
          { name: 'MUI X Date and Time Pickers' },
        ],
        moreLinks: ['https://mui.com/x/react-date-pickers/getting-started/'],
      }}
    />
  );
}
