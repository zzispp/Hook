'use client';

import type { TabProps } from '@mui/material/Tab';

import { Fragment } from 'react';
import { upperFirst } from 'es-toolkit';
import { useTabs } from 'minimal-shared/hooks';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Paper from '@mui/material/Paper';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const COLORS = ['inherit', 'primary', 'secondary'] as const;
const ICON_POSITIONS = ['top', 'bottom', 'start', 'end'] as const;

const TABS: TabProps[] = [
  {
    value: 'one',
    icon: <Iconify width={24} icon="solar:phone-bold" />,
    label: 'Item one',
  },
  {
    value: 'two',
    icon: <Iconify width={24} icon="solar:heart-bold" />,
    label: 'Item two',
  },
  {
    value: 'three',
    icon: <Iconify width={24} icon="solar:headphones-round-bold" />,
    label: 'Item three',
    disabled: true,
  },
  {
    value: 'four',
    icon: <Iconify width={24} icon="solar:headphones-round-bold" />,
    label: 'Item four',
  },
  {
    value: 'five',
    icon: <Iconify width={24} icon="solar:headphones-round-bold" />,
    label: 'Item five',
    disabled: true,
  },
  {
    value: 'six',
    icon: <Iconify width={24} icon="solar:headphones-round-bold" />,
    label: 'Item six',
  },
  {
    value: 'seven',
    icon: <Iconify width={24} icon="solar:headphones-round-bold" />,
    label: 'Item seven',
  },
];

// ----------------------------------------------------------------------

export function TabsView() {
  const basicTabs = useTabs('one');
  const customTabs = useTabs('one');
  const scrollableTabs = useTabs('one');

  const DEMO_COMPONENTS = [
    {
      name: 'Text',
      component: (
        <ComponentBox sx={{ flexDirection: 'column', alignItems: 'unset' }}>
          <Tabs value={basicTabs.value} onChange={basicTabs.onChange}>
            {TABS.slice(0, 3).map((tab) => (
              <Tab key={tab.value} value={tab.value} label={tab.label} disabled={tab.disabled} />
            ))}
          </Tabs>

          <Paper variant="outlined" sx={{ p: 3, typography: 'body2', borderRadius: 1.5 }}>
            {TABS.slice(0, 3).map((tab) =>
              tab.value === basicTabs.value ? (
                <Fragment key={tab.value}>{tab.label}</Fragment>
              ) : null
            )}
          </Paper>
        </ComponentBox>
      ),
    },
    {
      name: 'Icon',
      component: (
        <Box sx={contentStyles.grid()}>
          <ComponentBox title="Icon only">
            <Tabs value={basicTabs.value} onChange={basicTabs.onChange}>
              {TABS.slice(0, 3).map((tab) => (
                <Tab key={tab.value} icon={tab.icon} value={tab.value} />
              ))}
            </Tabs>
          </ComponentBox>

          {ICON_POSITIONS.map((position) => (
            <ComponentBox key={position} title={`${upperFirst(position)} icon`}>
              <Tabs value={basicTabs.value} onChange={basicTabs.onChange}>
                {TABS.slice(0, 3).map((tab) => (
                  <Tab
                    iconPosition={position}
                    key={tab.value}
                    icon={tab.icon}
                    label={tab.label}
                    value={tab.value}
                    disabled={tab.disabled}
                  />
                ))}
              </Tabs>
            </ComponentBox>
          ))}
        </Box>
      ),
    },
    {
      name: 'Colors',
      component: (
        <Box sx={contentStyles.grid()}>
          <ComponentBox title="Text color">
            {COLORS.map((color) => (
              <Tabs
                key={color}
                textColor={color}
                value={basicTabs.value}
                onChange={basicTabs.onChange}
              >
                {TABS.slice(0, 3).map((tab) => (
                  <Tab key={tab.value} icon={tab.icon} value={tab.value} disabled={tab.disabled} />
                ))}
              </Tabs>
            ))}
          </ComponentBox>

          <ComponentBox title="Indicator color">
            {COLORS.map((color) => (
              <Tabs
                key={color}
                indicatorColor={color}
                value={basicTabs.value}
                onChange={basicTabs.onChange}
              >
                {TABS.slice(0, 3).map((tab) => (
                  <Tab key={tab.value} icon={tab.icon} value={tab.value} disabled={tab.disabled} />
                ))}
              </Tabs>
            ))}
          </ComponentBox>

          <ComponentBox title="Text & indicator color">
            {COLORS.map((color) => (
              <Tabs
                key={color}
                textColor={color}
                indicatorColor={color}
                value={basicTabs.value}
                onChange={basicTabs.onChange}
              >
                {TABS.slice(0, 3).map((tab) => (
                  <Tab key={tab.value} icon={tab.icon} value={tab.value} disabled={tab.disabled} />
                ))}
              </Tabs>
            ))}
          </ComponentBox>
        </Box>
      ),
    },
    {
      name: 'Scrollable',
      component: (
        <ComponentBox>
          <Tabs
            value={scrollableTabs.value}
            onChange={scrollableTabs.onChange}
            sx={{ maxWidth: 320 }}
          >
            {TABS.map((tab) => (
              <Tab key={tab.value} label={tab.label} value={tab.value} />
            ))}
          </Tabs>
        </ComponentBox>
      ),
    },
    {
      name: 'Vertical',
      component: (
        <ComponentBox>
          <Paper variant="outlined" sx={{ display: 'flex', width: 1, minHeight: 320 }}>
            <Tabs
              orientation="vertical"
              value={basicTabs.value}
              onChange={basicTabs.onChange}
              sx={{ width: 200, borderRight: (theme) => `1px solid ${theme.palette.divider}` }}
            >
              {TABS.slice(0, 3).map((tab) => (
                <Tab key={tab.value} label={tab.label} value={tab.value} />
              ))}
            </Tabs>
            <Box sx={{ p: 3, typography: 'body2', flex: '1 1 auto' }}>
              {TABS.slice(0, 3).map((tab) =>
                tab.value === basicTabs.value ? (
                  <Fragment key={tab.value}>{tab.label}</Fragment>
                ) : null
              )}
            </Box>
          </Paper>
        </ComponentBox>
      ),
    },
    {
      name: 'Custom',
      component: (
        <Box sx={contentStyles.column()}>
          <ComponentBox title="Dynamic width">
            <Tabs
              indicatorColor="custom"
              value={customTabs.value}
              onChange={customTabs.onChange}
              sx={{ borderRadius: 1 }}
            >
              {TABS.slice(0, 4).map((tab) => (
                <Tab key={tab.value} value={tab.value} label={tab.label} disabled={tab.disabled} />
              ))}
            </Tabs>
          </ComponentBox>

          <ComponentBox title="Scrollable width">
            <Tabs
              variant="scrollable"
              textColor="secondary"
              indicatorColor="custom"
              value={customTabs.value}
              onChange={customTabs.onChange}
              sx={{ height: 72, maxWidth: 320, borderRadius: 1 }}
            >
              {TABS.slice(0, 4).map((tab) => (
                <Tab key={tab.value} value={tab.value} label={tab.label} disabled={tab.disabled} />
              ))}
            </Tabs>
          </ComponentBox>

          <ComponentBox title="Full width">
            <Tabs
              variant="fullWidth"
              textColor="primary"
              indicatorColor="custom"
              value={customTabs.value}
              onChange={customTabs.onChange}
              sx={{ '--indicator-radius': '12px', width: 1, height: 96, borderRadius: 2 }}
            >
              {TABS.slice(0, 4).map((tab) => (
                <Tab key={tab.value} value={tab.value} label={tab.label} disabled={tab.disabled} />
              ))}
            </Tabs>
          </ComponentBox>

          <ComponentBox title="Vertical">
            <Paper variant="outlined" sx={{ display: 'flex', width: 1, minHeight: 320 }}>
              <Tabs
                orientation="vertical"
                indicatorColor="custom"
                value={basicTabs.value}
                onChange={basicTabs.onChange}
                sx={{ width: 200, borderRight: (theme) => `1px solid ${theme.palette.divider}` }}
              >
                {TABS.slice(0, 3).map((tab) => (
                  <Tab key={tab.value} label={tab.label} value={tab.value} />
                ))}
              </Tabs>
              <Box sx={{ p: 3, typography: 'body2', flex: '1 1 auto' }}>
                {TABS.slice(0, 3).map((tab) =>
                  tab.value === basicTabs.value ? (
                    <Fragment key={tab.value}>{tab.label}</Fragment>
                  ) : null
                )}
              </Box>
            </Paper>
          </ComponentBox>
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Tabs',
        moreLinks: ['https://mui.com/material-ui/react-tabs/'],
      }}
    />
  );
}
