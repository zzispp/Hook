'use client';

import type { IconContainerProps } from '@mui/material/Rating';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Rating from '@mui/material/Rating';
import Tooltip from '@mui/material/Tooltip';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['xxSmall', 'xSmall', 'small', 'medium', 'large'] as const;

const LABELS: {
  [index: string]: string;
} = {
  0.5: 'Useless',
  1: 'Useless+',
  1.5: 'Poor',
  2: 'Poor+',
  2.5: 'Ok',
  3: 'Ok+',
  3.5: 'Good',
  4: 'Good+',
  4.5: 'Excellent',
  5: 'Excellent+',
};

const CUSTOM_ICONS: {
  [index: string]: {
    icon: React.ReactElement;
    label: string;
  };
} = {
  1: { icon: <Iconify icon="ic:round-sentiment-very-dissatisfied" />, label: 'Very Dissatisfied' },
  2: { icon: <Iconify icon="ic:round-sentiment-dissatisfied" />, label: 'Dissatisfied' },
  3: { icon: <Iconify icon="ic:round-sentiment-neutral" />, label: 'Neutral' },
  4: { icon: <Iconify icon="ic:round-sentiment-satisfied" />, label: 'Satisfied' },
  5: { icon: <Iconify icon="ic:round-sentiment-very-satisfied" />, label: 'Very Satisfied' },
};

// ----------------------------------------------------------------------

export function RatingView() {
  const [hover, setHover] = useState(-1);
  const [value, setValue] = useState<number | null>(2);

  const DEMO_COMPONENTS = [
    {
      name: 'Controlled',
      component: (
        <ComponentBox>
          <Rating
            name="simple-controlled"
            value={value}
            onChange={(event, newValue) => setValue(newValue)}
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Read only',
      component: (
        <ComponentBox>
          <Rating name="read-only" value={value} readOnly />
        </ComponentBox>
      ),
    },
    {
      name: 'Disabled',
      component: (
        <ComponentBox>
          <Rating name="disabled" value={value} disabled />
        </ComponentBox>
      ),
    },
    {
      name: 'Custom icon and color',
      component: (
        <ComponentBox>
          <Rating
            name="customized-color"
            defaultValue={2}
            getLabelText={(ratingValue) => `${ratingValue} Heart${ratingValue !== 1 ? 's' : ''}`}
            precision={0.5}
            icon={<Iconify icon="solar:heart-bold" />}
            emptyIcon={<Iconify icon="solar:heart-bold" />}
            sx={{ color: 'info.main', '&:hover': { color: 'info.dark' } }}
          />

          <Rating
            name="customized-icons"
            defaultValue={2}
            getLabelText={(ratingValue) => CUSTOM_ICONS[ratingValue].label}
            slotProps={{
              icon: { component: IconContainer },
            }}
          />
        </ComponentBox>
      ),
    },
    {
      name: '10 stars',
      component: (
        <ComponentBox>
          <Rating name="customized-10" defaultValue={2} max={10} />
        </ComponentBox>
      ),
    },
    {
      name: 'Hover feedback',
      component: (
        <ComponentBox>
          <Rating
            name="hover-feedback"
            value={value}
            precision={0.5}
            onChange={(event, newValue) => setValue(newValue)}
            onChangeActive={(event, newHover) => setHover(newHover)}
          />
          {value !== null && <Box sx={{ ml: 2 }}>{LABELS[hover !== -1 ? hover : value]}</Box>}
        </ComponentBox>
      ),
    },
    {
      name: 'Half ratings',
      component: (
        <ComponentBox>
          <Rating name="half-rating" defaultValue={2.5} precision={0.5} />
          <Rating name="half-rating-read" defaultValue={2.5} precision={0.5} readOnly />
        </ComponentBox>
      ),
    },
    {
      name: 'Sizes',
      component: (
        <ComponentBox>
          {SIZES.map((size) => (
            <Tooltip key={size} title={size}>
              <Rating name={`size-${size}`} defaultValue={2} size={size} />
            </Tooltip>
          ))}
        </ComponentBox>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Rating',
        moreLinks: ['https://mui.com/material-ui/react-rating/'],
      }}
    />
  );
}

// ----------------------------------------------------------------------

function IconContainer({ value, ...other }: IconContainerProps) {
  return <span {...other}>{CUSTOM_ICONS[value].icon}</span>;
}
