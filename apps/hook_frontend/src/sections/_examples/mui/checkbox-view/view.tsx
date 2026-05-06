'use client';

import { useState, useCallback } from 'react';

import Checkbox from '@mui/material/Checkbox';
import FormGroup from '@mui/material/FormGroup';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const PLACEMENTS = ['top', 'start', 'bottom', 'end'] as const;
const COLORS = ['default', ...colorKeys.palette] as const;

// ----------------------------------------------------------------------

export function CheckboxView() {
  const [checked, setChecked] = useState([true, false]);

  const handleChangeParent = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setChecked([event.target.checked, event.target.checked]);
  }, []);

  const handleChangeChild1 = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      setChecked([event.target.checked, checked[1]]);
    },
    [checked]
  );

  const handleChangeChild2 = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      setChecked([checked[0], event.target.checked]);
    },
    [checked]
  );

  const DEMO_COMPONENTS = [
    {
      name: 'Basic',
      component: (
        <ComponentBox>
          <Checkbox
            size="medium"
            slotProps={{
              input: {
                id: 'unchecked-checkbox',
                'aria-label': 'Unchecked checkbox',
              },
            }}
          />
          <Checkbox
            size="medium"
            defaultChecked
            slotProps={{
              input: {
                id: 'checked-checkbox',
                'aria-label': 'Checked checkbox',
              },
            }}
          />
          <Checkbox
            size="medium"
            defaultChecked
            indeterminate
            slotProps={{
              input: {
                id: 'indeterminate-checkbox',
                'aria-label': 'Indeterminate checkbox',
              },
            }}
          />
          <Checkbox
            size="medium"
            disabled
            slotProps={{
              input: {
                id: 'disabled-checkbox',
                'aria-label': 'Disabled checkbox',
              },
            }}
          />
          <Checkbox
            size="medium"
            disabled
            defaultChecked
            slotProps={{
              input: {
                id: 'disabled-checked-checkbox',
                'aria-label': 'Disabled checked checkbox',
              },
            }}
          />
          <Checkbox
            size="medium"
            disabled
            indeterminate
            slotProps={{
              input: {
                id: 'disabled-indeterminate-checkbox',
                'aria-label': 'Disabled indeterminate checkbox',
              },
            }}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Sizes',
      component: (
        <ComponentBox>
          <FormControlLabel
            label="Medium"
            control={
              <Checkbox
                size="medium"
                defaultChecked
                slotProps={{
                  input: {
                    id: 'medium-size-checkbox',
                  },
                }}
              />
            }
          />
          <FormControlLabel
            label="Small"
            control={
              <Checkbox
                size="small"
                defaultChecked
                slotProps={{
                  input: {
                    id: 'small-size-checkbox',
                  },
                }}
              />
            }
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Custom icons',
      component: (
        <ComponentBox>
          <FormControlLabel
            label="Custom icon"
            control={
              <Checkbox
                color="info"
                size="small"
                icon={<Iconify icon="solar:heart-bold" />}
                checkedIcon={<Iconify icon="solar:heart-bold" />}
                slotProps={{
                  input: {
                    id: 'favorite-checkbox',
                  },
                }}
              />
            }
          />

          <FormControlLabel
            label="Custom icon"
            control={
              <Checkbox
                color="error"
                size="small"
                icon={<Iconify icon="eva:award-fill" />}
                checkedIcon={<Iconify icon="eva:award-fill" />}
                slotProps={{
                  input: {
                    id: 'award-checkbox',
                  },
                }}
              />
            }
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Placement',
      component: (
        <ComponentBox>
          <FormControl component="fieldset">
            <FormGroup aria-label="position" row>
              {PLACEMENTS.map((placement) => (
                <FormControlLabel
                  key={placement}
                  value={placement}
                  label={placement}
                  labelPlacement={placement}
                  control={
                    <Checkbox
                      size="medium"
                      slotProps={{
                        input: {
                          id: `${placement}-checkbox`,
                        },
                      }}
                    />
                  }
                  sx={{ textTransform: 'capitalize' }}
                />
              ))}
            </FormGroup>
          </FormControl>
        </ComponentBox>
      ),
    },
    {
      name: 'Colors',
      component: (
        <ComponentBox>
          <FormGroup>
            {COLORS.map((color) => (
              <FormControlLabel
                key={color}
                label={color}
                control={
                  <Checkbox
                    size="medium"
                    defaultChecked
                    color={color}
                    slotProps={{
                      input: {
                        id: `${color}-checkbox`,
                      },
                    }}
                  />
                }
                sx={{ textTransform: 'capitalize' }}
              />
            ))}

            <FormControlLabel
              disabled
              label="Disabled"
              control={
                <Checkbox
                  size="medium"
                  defaultChecked
                  color="error"
                  slotProps={{
                    input: {
                      id: 'color-disabled-checkbox',
                    },
                  }}
                />
              }
            />
          </FormGroup>

          <FormControl component="fieldset">
            <FormGroup>
              {COLORS.map((color) => (
                <FormControlLabel
                  key={color}
                  label={color}
                  control={
                    <Checkbox
                      size="medium"
                      defaultChecked
                      indeterminate
                      color={color}
                      slotProps={{
                        input: {
                          id: `${color}-indeterminate-checkbox`,
                        },
                      }}
                    />
                  }
                  sx={{ textTransform: 'capitalize' }}
                />
              ))}

              <FormControlLabel
                disabled
                label="Disabled"
                control={
                  <Checkbox
                    size="medium"
                    defaultChecked
                    indeterminate
                    color="error"
                    slotProps={{
                      input: {
                        id: 'color-disabled-indeterminate-checkbox',
                      },
                    }}
                  />
                }
              />
            </FormGroup>
          </FormControl>
        </ComponentBox>
      ),
    },
    {
      name: 'Indeterminate',
      component: (
        <ComponentBox>
          <div>
            <FormControlLabel
              label="Parent"
              control={
                <Checkbox
                  size="medium"
                  checked={checked[0] && checked[1]}
                  indeterminate={checked[0] !== checked[1]}
                  onChange={handleChangeParent}
                  slotProps={{
                    input: {
                      id: 'Parent-checkbox',
                    },
                  }}
                />
              }
            />
            <div>
              <FormControlLabel
                label="Child 1"
                control={
                  <Checkbox
                    size="medium"
                    checked={checked[0]}
                    onChange={handleChangeChild1}
                    slotProps={{
                      input: {
                        id: 'child-1-checkbox',
                      },
                    }}
                  />
                }
              />
              <FormControlLabel
                label="Child 2"
                control={
                  <Checkbox
                    size="medium"
                    checked={checked[1]}
                    onChange={handleChangeChild2}
                    slotProps={{
                      input: {
                        id: 'child-2-checkbox',
                      },
                    }}
                  />
                }
              />
            </div>
          </div>
        </ComponentBox>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Checkbox',
        moreLinks: ['https://mui.com/material-ui/react-checkbox/'],
      }}
    />
  );
}
