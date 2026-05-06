'use client';

import { ChipVariant } from './chip-variant';
import { ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  { name: 'Filled', component: <ChipVariant variant="filled" /> },
  { name: 'Outlined', component: <ChipVariant variant="outlined" /> },
  { name: 'Soft', component: <ChipVariant variant="soft" /> },
];

// ----------------------------------------------------------------------

export function ChipView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Chip',
        moreLinks: ['https://mui.com/material-ui/react-chip/'],
      }}
    />
  );
}
