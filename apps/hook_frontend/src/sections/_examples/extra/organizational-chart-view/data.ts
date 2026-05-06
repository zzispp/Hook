import type { Theme, SxProps } from '@mui/material/styles';
import type { OrgChartBaseNode } from 'src/components/organizational-chart';

import { _mock } from 'src/_mock';

// ----------------------------------------------------------------------

export type NodeProps = OrgChartBaseNode & {
  id?: string;
  name: string;
  group?: string;
  role?: string;
  avatarUrl?: string;
  children?: any;
  sx?: SxProps<Theme>;
};

const rootNode: NodeProps = {
  group: 'root',
  role: 'CEO, Co-founder',
  name: _mock.fullName(1),
  avatarUrl: _mock.image.avatar(1),
};

const group = {
  product: 'product design',
  development: 'development',
  marketing: 'marketing',
};

const createNode = (
  index: number,
  role?: string,
  groupName?: string,
  children?: NodeProps[]
): NodeProps => ({
  id: _mock.id(index),
  name: _mock.fullName(index),
  avatarUrl: _mock.image.avatar(index),
  role,
  group: groupName,
  children,
});

// ----------------------------------------------------------------------

export const SIMPLE_DATA = {
  ...rootNode,
  children: [
    createNode(2, 'Lead', undefined, [createNode(3, 'Senior')]),
    createNode(4, 'Lead', undefined, [
      createNode(5, 'Senior', undefined, [
        createNode(6, 'Back end developer', undefined, [createNode(7, 'Back end developer')]),
        createNode(8, 'Front end'),
      ]),
    ]),
    createNode(9, 'Lead', undefined, [createNode(10, 'Support'), createNode(11, 'Content writer')]),
  ],
} satisfies NodeProps;

// ----------------------------------------------------------------------

export const GROUP_DATA = {
  ...rootNode,
  children: [
    {
      name: group.product,
      group: group.product,
      children: [createNode(2, 'Lead', group.product, [createNode(3, 'Senior', group.product)])],
    },
    {
      name: group.development,
      group: group.development,
      children: [
        createNode(4, 'Lead', group.development, [
          createNode(5, 'Senior', group.development, [
            createNode(6, 'Back end developer', group.development, [
              createNode(7, 'Back end developer', group.development),
            ]),
            createNode(8, 'Front end', group.development),
          ]),
        ]),
      ],
    },
    {
      name: group.marketing,
      group: group.marketing,
      children: [
        createNode(9, 'Lead', group.marketing, [
          createNode(10, 'Lead', group.marketing),
          createNode(11, 'Content writer', group.marketing),
        ]),
      ],
    },
  ],
} satisfies NodeProps;
