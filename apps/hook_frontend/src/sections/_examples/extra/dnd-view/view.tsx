'use client';

import GlobalStyles from '@mui/material/GlobalStyles';

import { ComponentLayout } from '../../layout';
import { SortableList } from './sortable-list';
import { SortableGrid } from './sortable-grid';

// ----------------------------------------------------------------------

const initialItems = Array.from({ length: 12 }, (_, index) => ({
  id: `id-${index + 1}`,
  name: `${index + 1}`,
}));

const DEMO_COMPONENTS = [
  { name: 'Grid', component: <SortableGrid data={initialItems} /> },
  {
    name: 'Vertical',
    component: <SortableList data={initialItems} orientation="vertical" />,
  },
  {
    name: 'Horizontal',
    component: <SortableList data={initialItems} orientation="horizontal" indicatorShape="line" />,
  },
];

const inputGlobalStyles = () => (
  <GlobalStyles
    styles={{
      body: {
        '--dnd-item-gap': '20px',
        '--dnd-item-radius': '12px',
      },
    }}
  />
);

// ----------------------------------------------------------------------

export function DndView() {
  return (
    <>
      {inputGlobalStyles()}

      <ComponentLayout
        sectionData={DEMO_COMPONENTS}
        heroProps={{
          heading: 'Dnd',
          moreLinks: ['https://atlassian.design/components/pragmatic-drag-and-drop/examples'],
        }}
      />
    </>
  );
}
