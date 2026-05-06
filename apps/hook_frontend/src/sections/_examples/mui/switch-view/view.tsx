'use client';

import Box from '@mui/material/Box';
import Switch from '@mui/material/Switch';
import FormGroup from '@mui/material/FormGroup';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';

import { colorKeys } from 'src/theme/core';

import { ComponentBox, contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const PLACEMENTS = ['top', 'start', 'bottom', 'end'] as const;
const COLORS = ['default', ...colorKeys.palette] as const;

// ----------------------------------------------------------------------

export function SwitchView() {
  const renderColor = (title: string, disabled?: boolean) => (
    <ComponentBox title={title}>
      <FormControl component="fieldset">
        <FormGroup>
          {COLORS.map((color) => (
            <FormControlLabel
              key={color}
              label={color}
              disabled={disabled}
              control={
                <Switch
                  defaultChecked
                  color={color}
                  slotProps={{ input: { id: `${color}-${title}-switch` } }}
                />
              }
              sx={{ textTransform: 'capitalize' }}
            />
          ))}
        </FormGroup>

        <FormControlLabel
          disabled={disabled}
          label="No check"
          control={
            <Switch color="error" slotProps={{ input: { id: `${title}-disabled-off-switch` } }} />
          }
        />
      </FormControl>
    </ComponentBox>
  );

  const DEMO_COMPONENTS = [
    {
      name: 'Basic',
      component: (
        <ComponentBox>
          <Switch defaultChecked slotProps={{ input: { id: 'checked-switch' } }} />
          <Switch slotProps={{ input: { id: 'unchecked-switch' } }} />
          <Switch disabled slotProps={{ input: { id: 'disabled-unchecked-switch' } }} />
          <Switch disabled checked slotProps={{ input: { id: 'disabled-checked-switch' } }} />
          <Switch
            defaultChecked
            color="default"
            slotProps={{ input: { id: 'color-checked-switch' } }}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Sizes',
      component: (
        <ComponentBox>
          <FormGroup row>
            <FormControlLabel
              label="Small"
              control={<Switch size="small" slotProps={{ input: { id: 'small-switch' } }} />}
            />
            <FormControlLabel
              label="Medium"
              control={<Switch slotProps={{ input: { id: 'medium-switch' } }} />}
            />
          </FormGroup>
        </ComponentBox>
      ),
    },
    {
      name: 'Placements',
      component: (
        <ComponentBox>
          <FormGroup row>
            {PLACEMENTS.map((placement) => (
              <FormControlLabel
                key={placement}
                value={placement}
                label={placement}
                labelPlacement={placement}
                control={<Switch slotProps={{ input: { id: `${placement}-switch` } }} />}
                sx={{ textTransform: 'capitalize' }}
              />
            ))}
          </FormGroup>
        </ComponentBox>
      ),
    },
    {
      name: 'Colors',
      component: (
        <Box sx={contentStyles.grid()}>
          {renderColor('Default')}
          {renderColor('Disabled', true)}
        </Box>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Switch',
        moreLinks: ['https://mui.com/material-ui/react-switch/'],
      }}
    />
  );
}
