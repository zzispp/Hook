'use client';

import { CustomIcons } from './custom-icon';
import { CustomStyling } from './custom-styling';
import { BasicRichTree, BasicSimpleTree } from './basic';
import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Simple tree view',
    component: (
      <ComponentBox>
        <BasicSimpleTree />
      </ComponentBox>
    ),
  },
  {
    name: 'Rich tree view',
    component: (
      <ComponentBox>
        <BasicRichTree />
      </ComponentBox>
    ),
  },
  {
    name: 'Custom icons',
    component: (
      <ComponentBox>
        <CustomIcons />
      </ComponentBox>
    ),
  },
  {
    name: 'Checkbox selection',
    component: (
      <ComponentBox>
        <BasicSimpleTree multiSelect checkboxSelection />
      </ComponentBox>
    ),
  },
  {
    name: 'Custom styling',
    component: (
      <ComponentBox>
        <CustomStyling />
      </ComponentBox>
    ),
  },
];

// ----------------------------------------------------------------------

export function TreeView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'MUI X Tree View',
        moreLinks: ['https://mui.com/x/react-tree-view/'],
      }}
    />
  );
}
