'use client';

import { varAlpha } from 'minimal-shared/utils';

import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import AlertTitle from '@mui/material/AlertTitle';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const SEVERITY = ['info', 'success', 'warning', 'error'] as const;

// ----------------------------------------------------------------------

const DEMO_COMPONENTS = [
  {
    name: 'Standard',
    component: (
      <ComponentBox>
        {SEVERITY.map((color) => (
          <Alert key={color} severity={color} onClose={() => {}} sx={{ width: 1 }}>
            This is an {color} alert — check it out!
          </Alert>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Filled',
    component: (
      <ComponentBox>
        {SEVERITY.map((color) => (
          <Alert key={color} severity={color} variant="filled" onClose={() => {}} sx={{ width: 1 }}>
            This is an {color} alert — check it out!
          </Alert>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Outlined',
    component: (
      <ComponentBox>
        {SEVERITY.map((color) => (
          <Alert
            key={color}
            severity={color}
            variant="outlined"
            onClose={() => {}}
            sx={{ width: 1 }}
          >
            This is an {color} alert — check it out!
          </Alert>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Description',
    component: (
      <ComponentBox>
        {SEVERITY.map((color) => (
          <Alert key={color} severity={color} onClose={() => {}} sx={{ width: 1 }}>
            <AlertTitle sx={{ textTransform: 'capitalize' }}> {color} </AlertTitle>
            This is an {color} alert — <strong>check it out!</strong>
          </Alert>
        ))}
      </ComponentBox>
    ),
  },
  {
    name: 'Actions',
    component: (
      <ComponentBox>
        <Alert
          severity="info"
          action={
            <Button color="info" size="small" variant="soft">
              Action
            </Button>
          }
          sx={{ width: 1 }}
        >
          This is an info alert — check it out!
        </Alert>

        <Alert
          severity="info"
          variant="filled"
          action={
            <>
              <Button
                color="inherit"
                size="small"
                variant="outlined"
                sx={[
                  (theme) => ({
                    mr: 1,
                    border: `1px solid ${varAlpha(theme.vars.palette.common.whiteChannel, 0.48)}`,
                  }),
                ]}
              >
                Undo
              </Button>
              <Button size="small" color="info" variant="contained" sx={{ bgcolor: 'info.dark' }}>
                Action
              </Button>
            </>
          }
          sx={{ width: 1 }}
        >
          This is an info alert — check it out!
        </Alert>

        <Alert
          severity="info"
          variant="outlined"
          action={
            <>
              <Button color="info" size="small" variant="outlined" sx={{ mr: 1 }}>
                Undo
              </Button>
              <Button color="info" size="small" variant="contained" sx={{ bgcolor: 'info.dark' }}>
                Action
              </Button>
            </>
          }
          sx={{ width: 1 }}
        >
          This is an info alert — check it out!
        </Alert>
      </ComponentBox>
    ),
  },
];

// ----------------------------------------------------------------------

export function AlertView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Alert',
        moreLinks: ['https://mui.com/material-ui/react-alert/'],
      }}
    />
  );
}
