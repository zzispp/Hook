'use client';

import { useState, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Slider from '@mui/material/Slider';
import Divider from '@mui/material/Divider';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium'] as const;
const COLORS = ['inherit', ...colorKeys.palette] as const;

const PRICES = [
  { value: 0, label: '$0' },
  { value: 25, label: '250' },
  { value: 50, label: '500' },
  { value: 75, label: '750' },
  { value: 100, label: '1000' },
];

const MARKS = [
  { value: 0, label: '0°C' },
  { value: 20, label: '20°C' },
  { value: 37, label: '37°C' },
  { value: 100, label: '100°C' },
];

// ----------------------------------------------------------------------

function valuePrice(value: number) {
  return value > 0 ? `$${value}0` : `${value}`;
}

function valueLabelFormatPrice(value: number) {
  return value > 0 ? `$${value}` : value;
}

function getAriaValueText(value: number) {
  return `$${value}°C`;
}

function valueLabelFormat(value: number) {
  return MARKS.findIndex((mark) => mark.value === value) + 1;
}

// ----------------------------------------------------------------------

export function SliderView() {
  const [value, setValue] = useState<number>(30);
  const [price, setPrice] = useState<number[]>([20, 37]);

  const handleChangeValue = useCallback((event: Event, newValue: number | number[]) => {
    setValue(newValue as number);
  }, []);

  const handleChangePrice = useCallback((event: Event, newValue: number | number[]) => {
    setPrice(newValue as number[]);
  }, []);

  const DEMO_COMPONENTS = [
    {
      name: 'Volume',
      component: (
        <ComponentBox sx={{ flexWrap: 'unset' }}>
          <Iconify width={24} icon="solar:volume-bold" />
          <Slider value={value} onChange={handleChangeValue} aria-labelledby="continuous-slider" />
          <Iconify width={24} icon="solar:volume-loud-bold" />
        </ComponentBox>
      ),
    },
    {
      name: 'Disabled',
      component: (
        <ComponentBox>
          <Slider disabled defaultValue={30} />
        </ComponentBox>
      ),
    },
    {
      name: 'Temperature',
      component: (
        <ComponentBox>
          <Slider
            marks
            min={10}
            max={110}
            step={10}
            defaultValue={30}
            valueLabelDisplay="auto"
            getAriaValueText={getAriaValueText}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Sizes',
      component: (
        <ComponentBox>
          {SIZES.map((size) => (
            <Slider
              key={size}
              marks
              min={10}
              max={110}
              step={10}
              size={size}
              defaultValue={30}
              valueLabelDisplay="auto"
              getAriaValueText={getAriaValueText}
            />
          ))}
        </ComponentBox>
      ),
    },
    {
      name: 'Small steps',
      component: (
        <ComponentBox>
          <Slider
            marks
            min={-0.00000005}
            max={0.0000001}
            step={0.00000001}
            valueLabelDisplay="auto"
            defaultValue={0.00000005}
            getAriaValueText={getAriaValueText}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Custom marks',
      component: (
        <ComponentBox>
          <Slider
            step={10}
            marks={MARKS}
            defaultValue={20}
            valueLabelDisplay="auto"
            getAriaValueText={getAriaValueText}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Restricted values',
      component: (
        <ComponentBox>
          <Slider
            step={null}
            marks={MARKS}
            defaultValue={20}
            valueLabelDisplay="auto"
            valueLabelFormat={valueLabelFormat}
            getAriaValueText={getAriaValueText}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Range',
      component: (
        <ComponentBox sx={{ flexDirection: 'column' }}>
          <Slider
            step={10}
            value={price}
            marks={PRICES}
            scale={(x) => x * 10}
            valueLabelDisplay="on"
            onChange={handleChangePrice}
            getAriaValueText={valuePrice}
            valueLabelFormat={valueLabelFormatPrice}
          />

          <Box
            sx={(theme) => ({
              p: 2,
              gap: 2,
              borderRadius: 1,
              display: 'flex',
              typography: 'subtitle2',
              bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.12),
            })}
          >
            <Box component="span">Min: {valuePrice(price[0])}</Box>
            <Divider orientation="vertical" flexItem />
            <Box component="span">Max: {valuePrice(price[1])}</Box>
          </Box>
        </ComponentBox>
      ),
    },
    {
      name: 'Vertical',
      component: (
        <ComponentBox sx={{ height: 360, gap: 10 }}>
          {SIZES.map((size) => (
            <Slider
              key={size}
              size={size}
              marks={MARKS}
              orientation="vertical"
              defaultValue={[12, 72]}
              valueLabelDisplay="auto"
              getAriaLabel={() => 'Temperature'}
              getAriaValueText={getAriaValueText}
              color={size === 'small' ? 'inherit' : 'primary'}
            />
          ))}
        </ComponentBox>
      ),
    },
    {
      name: 'Colors',
      component: (
        <ComponentBox sx={{ flexDirection: 'column' }}>
          {COLORS.map((color) => (
            <Slider
              key={color}
              marks
              min={10}
              max={110}
              step={10}
              color={color}
              defaultValue={30}
              valueLabelDisplay="auto"
              getAriaValueText={getAriaValueText}
            />
          ))}
        </ComponentBox>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Slider',
        moreLinks: ['https://mui.com/material-ui/react-slider/'],
      }}
    />
  );
}
