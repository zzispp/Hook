import { Fragment, useCallback } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Avatar from '@mui/material/Avatar';

import { _mock } from 'src/_mock';
import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, contentStyles } from '../../layout';

// ----------------------------------------------------------------------

const COLORS = ['default', ...colorKeys.palette, ...colorKeys.common] as const;

// ----------------------------------------------------------------------

type Props = {
  variant: 'filled' | 'outlined' | 'soft';
};

export function ChipVariant({ variant }: Props) {
  const handleDelete = useCallback(() => {
    console.info('You clicked the delete icon.');
  }, []);

  return (
    <Box sx={contentStyles.grid()}>
      <ComponentBox title="Colors">
        {COLORS.map((color) => (
          <Fragment key={color}>
            <Chip
              clickable
              color={color}
              variant={variant}
              label="Clickable"
              avatar={<Avatar>M</Avatar>}
            />
            <Chip
              color={color}
              variant={variant}
              label="Deletable"
              avatar={<Avatar>M</Avatar>}
              onDelete={handleDelete}
            />
          </Fragment>
        ))}
      </ComponentBox>

      <Box sx={contentStyles.column()}>
        <ComponentBox title="Custom icons">
          <Chip
            variant={variant}
            label="Custom icon"
            onDelete={handleDelete}
            icon={<Iconify width={24} icon="eva:smiling-face-fill" />}
            deleteIcon={<Iconify icon="eva:checkmark-fill" />}
          />

          <Chip
            color="info"
            variant={variant}
            label="Custom icon"
            onDelete={handleDelete}
            avatar={<Avatar>M</Avatar>}
            deleteIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
          />
        </ComponentBox>

        <ComponentBox title="Disabled">
          <Chip
            disabled
            variant={variant}
            label="Disabled"
            onDelete={handleDelete}
            icon={<Iconify width={24} icon="eva:smiling-face-fill" />}
          />
          <Chip
            disabled
            color="error"
            variant={variant}
            label="Disabled"
            onDelete={handleDelete}
            avatar={<Avatar alt={_mock.fullName(1)} src={_mock.image.avatar(1)} />}
          />
          <Chip
            disabled
            color="info"
            variant={variant}
            label="Disabled"
            onDelete={handleDelete}
            avatar={<Avatar>M</Avatar>}
          />
        </ComponentBox>

        <ComponentBox title="Sizes">
          <Chip
            color="info"
            size="small"
            variant={variant}
            label="Small"
            onDelete={handleDelete}
            avatar={<Avatar>M</Avatar>}
          />
          <Chip
            color="info"
            variant={variant}
            label="Medium"
            onDelete={handleDelete}
            icon={<Iconify width={24} icon="eva:smiling-face-fill" />}
          />
        </ComponentBox>
      </Box>
    </Box>
  );
}
