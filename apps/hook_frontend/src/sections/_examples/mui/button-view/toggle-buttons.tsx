import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Divider from '@mui/material/Divider';
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { colorKeys } from 'src/theme/core';

import { Iconify } from 'src/components/iconify';

import { ComponentBox, contentStyles } from '../../layout';

// ----------------------------------------------------------------------

const SIZES = ['small', 'medium', 'large'] as const;
const COLORS = ['standard', ...colorKeys.palette] as const;

const viewContent = () => [
  <ToggleButton key="list" value="list">
    <Iconify icon="ic:round-view-list" />
  </ToggleButton>,
  <ToggleButton key="module" value="module">
    <Iconify icon="ic:round-view-module" />
  </ToggleButton>,
  <ToggleButton key="quilt" value="quilt">
    <Iconify icon="ic:round-view-quilt" />
  </ToggleButton>,
];

const alignContent = () => [
  <ToggleButton key="left" value="left">
    <Iconify icon="ic:round-format-align-left" />
  </ToggleButton>,
  <ToggleButton key="center" value="center">
    <Iconify icon="ic:round-format-align-center" />
  </ToggleButton>,
  <ToggleButton key="right" value="right">
    <Iconify icon="ic:round-format-align-right" />
  </ToggleButton>,
  <ToggleButton key="justify" value="justify" disabled>
    <Iconify icon="ic:round-format-align-justify" />
  </ToggleButton>,
];

const formatContent = () => [
  <ToggleButton key="bold" value="bold">
    <Iconify icon="ic:round-format-bold" />
  </ToggleButton>,
  <ToggleButton key="italic" value="italic">
    <Iconify icon="ic:round-format-italic" />
  </ToggleButton>,
  <ToggleButton key="underlined" value="underlined">
    <Iconify icon="ic:round-format-underlined" />
  </ToggleButton>,
  <ToggleButton key="color" value="color" disabled>
    <Iconify icon="ic:baseline-format-color-fill" />
    <Iconify icon="ic:baseline-arrow-drop-down" />
  </ToggleButton>,
];

// ----------------------------------------------------------------------

export function ToggleButtons() {
  const [view, setView] = useState('list');
  const [selected, setSelected] = useState(true);
  const [formats, setFormats] = useState(() => ['bold', 'italic']);
  const [alignment, setAlignment] = useState<string | null>('left');

  const handleChangeAlignment = useCallback(
    (event: React.MouseEvent<HTMLElement>, newValue: string | null) => {
      setAlignment(newValue);
    },
    []
  );

  const handleChangeFormat = useCallback(
    (event: React.MouseEvent<HTMLElement>, newValue: string[]) => {
      setFormats(newValue);
    },
    []
  );

  const handleChangeColor = useCallback(
    (event: React.MouseEvent<HTMLElement>, newValue: string) => {
      setView(newValue);
    },
    []
  );

  return (
    <>
      <Box sx={contentStyles.grid()}>
        <ComponentBox title="Exclusive selection">
          <ToggleButtonGroup exclusive value={alignment} onChange={handleChangeAlignment}>
            {alignContent()}
          </ToggleButtonGroup>
        </ComponentBox>

        <ComponentBox title="Multiple selection">
          <ToggleButtonGroup value={formats} onChange={handleChangeFormat}>
            {formatContent()}
          </ToggleButtonGroup>
        </ComponentBox>
      </Box>

      <Box sx={contentStyles.grid()}>
        <ComponentBox title="Sizes">
          <Box sx={contentStyles.row()}>
            {SIZES.map((size, index) => (
              <ToggleButton key={size} size={size} value="check">
                <Iconify icon="eva:checkmark-fill" width={20 + index * 2} />
              </ToggleButton>
            ))}
          </Box>

          {SIZES.map((size) => (
            <ToggleButtonGroup
              exclusive
              key={size}
              size={size}
              value={alignment}
              onChange={handleChangeAlignment}
            >
              {alignContent()}
            </ToggleButtonGroup>
          ))}
        </ComponentBox>

        <ComponentBox title="Disabled">
          <Box sx={contentStyles.row()}>
            <ToggleButton value="check" disabled>
              <Iconify icon="eva:checkmark-fill" />
            </ToggleButton>
            <ToggleButton value="check" disabled selected>
              <Iconify icon="eva:checkmark-fill" />
            </ToggleButton>
          </Box>

          <ToggleButtonGroup exclusive value="left">
            {alignContent()}
          </ToggleButtonGroup>
          <ToggleButtonGroup exclusive disabled value="left">
            {alignContent()}
          </ToggleButtonGroup>
        </ComponentBox>
      </Box>

      <ComponentBox title="Colors">
        {COLORS.map((color) => (
          <ToggleButtonGroup
            exclusive
            key={color}
            color={color}
            value={view}
            onChange={handleChangeColor}
          >
            {viewContent()}
          </ToggleButtonGroup>
        ))}

        <Divider sx={{ my: 2, width: 1, borderStyle: 'dashed' }} />

        {COLORS.map((color) => (
          <ToggleButton
            key={color}
            color={color}
            value="check"
            selected={selected}
            onChange={() => {
              setSelected(!selected);
            }}
          >
            <Iconify icon="eva:checkmark-fill" />
          </ToggleButton>
        ))}
      </ComponentBox>

      <ComponentBox title="Vertical & standalone">
        <ToggleButtonGroup
          exclusive
          orientation="vertical"
          value={view}
          onChange={handleChangeColor}
        >
          {viewContent()}
        </ToggleButtonGroup>

        <ToggleButton value="check" selected={selected} onChange={() => setSelected(!selected)}>
          <Iconify icon="eva:checkmark-fill" /> Label
        </ToggleButton>
      </ComponentBox>
    </>
  );
}
