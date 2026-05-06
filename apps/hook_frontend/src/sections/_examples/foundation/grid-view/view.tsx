'use client';

import { useState, useCallback } from 'react';

import Grid from '@mui/material/Grid';
import Radio from '@mui/material/Radio';
import Paper from '@mui/material/Paper';
import { useTheme } from '@mui/material/styles';
import RadioGroup from '@mui/material/RadioGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const COLUMNS = [12, 6, 4, 3, 2, 1];
const SPACINGS = Array.from({ length: 6 }, (_, index) => index);

// ----------------------------------------------------------------------

export function GridView() {
  const theme = useTheme();

  const [column, setColumn] = useState<number>(3);
  const [spacing, setSpacing] = useState<number>(2);

  const handleChangeSpacing = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setSpacing(Number(event.target.value));
  }, []);

  const handleChangeColumn = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setColumn(Number(event.target.value));
  }, []);

  const renderCols = (gridSize: number, gridSpacing: number) => (
    <Grid container spacing={gridSpacing} sx={{ width: 1 }}>
      {Array.from({ length: 12 }, (_, index) => (
        <Grid key={index} size={gridSize}>
          <Paper
            sx={{
              py: 3,
              textAlign: 'center',
              typography: 'subtitle1',
              color: theme.palette.text.disabled,
              boxShadow: theme.vars.customShadows.z8,
            }}
          >
            {index + 1}
          </Paper>
        </Grid>
      ))}
    </Grid>
  );

  const DEMO_COMPONENTS = [
    {
      name: 'Spacing',
      component: (
        <ComponentBox sx={{ flexDirection: 'column' }}>
          <RadioGroup
            row
            name="spacing"
            value={spacing}
            onChange={handleChangeSpacing}
            sx={{ mt: 3, display: 'flex', justifyContent: 'center' }}
          >
            {SPACINGS.map((value) => (
              <FormControlLabel
                key={value}
                value={value}
                label={`${value * 8}px`}
                control={<Radio />}
              />
            ))}
          </RadioGroup>

          {renderCols(1, spacing)}
        </ComponentBox>
      ),
    },
    {
      name: 'Column',
      component: (
        <ComponentBox>
          <RadioGroup
            row
            name="column"
            value={column}
            onChange={handleChangeColumn}
            sx={{ display: 'flex', justifyContent: 'center' }}
          >
            {COLUMNS.map((value, index) => (
              <FormControlLabel
                key={value}
                value={value}
                label={`${[...COLUMNS].reverse()[index]} col`}
                control={<Radio />}
              />
            ))}
          </RadioGroup>

          {renderCols(column, 2)}
        </ComponentBox>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Grid',
      }}
    />
  );
}
