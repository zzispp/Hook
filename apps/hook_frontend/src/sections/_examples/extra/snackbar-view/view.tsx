'use client';

import type { ButtonProps } from '@mui/material/Button';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { Iconify } from 'src/components/iconify';
import { toast, snackbarClasses } from 'src/components/snackbar';

import { ComponentBox, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const POSITIONS = [
  'top-left',
  'top-center',
  'top-right',
  'bottom-left',
  'bottom-center',
  'bottom-right',
] as const;

// ----------------------------------------------------------------------

export function SnackbarView() {
  const handlePromise = async () => {
    const promise = new Promise((resolve) => setTimeout(resolve, 3000));

    try {
      toast.promise(promise, {
        loading: 'Loading...',
        success: () => `Loading success!`,
        error: 'Error',
        closeButton: false,
      });

      await promise;
    } catch (error) {
      console.error(error);
    }
  };

  const DEMO_COMPONENTS = [
    {
      name: 'Default',
      component: (
        <ComponentBox>
          <ToastButton label="Default" onClick={() => toast('Event has been created')} />
          <ToastButton
            label="Message"
            onClick={() =>
              toast.message('Event has been created', {
                description: 'Monday, January 3rd at 6:00pm',
              })
            }
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Status',
      component: (
        <ComponentBox>
          <ToastButton
            label="Info"
            color="info"
            onClick={() => toast.info('Be at the area 10 minutes before the event time')}
          />
          <ToastButton
            label="Success"
            color="success"
            onClick={() => toast.success('Event has been created')}
          />
          <ToastButton
            label="Warning"
            color="warning"
            onClick={() => toast.warning('Event start time cannot be earlier than 8am')}
          />
          <ToastButton
            label="Error"
            color="error"
            onClick={() => toast.error('Event has not been created')}
          />
          <ToastButton label="Promise" variant="outlined" onClick={handlePromise} />
        </ComponentBox>
      ),
    },
    {
      name: 'Custom',
      component: (
        <ComponentBox>
          <ToastButton
            label="Action"
            variant="outlined"
            onClick={() =>
              toast('Event has been created', {
                closeButton: false,
                action: {
                  label: 'Undo',
                  onClick: () => console.log('Undo'),
                },
              })
            }
          />
          <ToastButton
            label="Custom action"
            variant="outlined"
            onClick={() =>
              toast.warning('Title', {
                id: 'action-id',
                closeButton: false,
                description: 'Description',
                action: (
                  <Box sx={{ display: 'flex' }}>
                    <Button size="small" color="warning" onClick={() => console.info('Action!')}>
                      Action
                    </Button>
                    <Button
                      size="small"
                      onClick={() => {
                        console.info('Dismissed');
                        toast.dismiss('action-id');
                      }}
                    >
                      Dismiss
                    </Button>
                  </Box>
                ),
              })
            }
          />
          <ToastButton
            label="Custom"
            variant="outlined"
            onClick={() =>
              toast(<Box component="span">Event has been created</Box>, {
                icon: (
                  <Iconify width={24} icon="solar:bell-bing-bold" sx={{ color: 'info.main' }} />
                ),
                description: <Box component="span">Monday, January 3rd at 6:00pm</Box>,
                classNames: { icon: snackbarClasses.unset },
                style: { background: 'white', color: 'inherit' },
              })
            }
          />
        </ComponentBox>
      ),
    },
    {
      name: 'Anchor origin',
      component: (
        <ComponentBox>
          {POSITIONS.map((position) => (
            <Button
              key={position}
              color="inherit"
              variant="outlined"
              onClick={() => toast(position, { position })}
            >
              {position}
            </Button>
          ))}
        </ComponentBox>
      ),
    },
  ];

  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Snackbar',
        moreLinks: ['https://sonner.emilkowal.ski/'],
      }}
    />
  );
}

// ----------------------------------------------------------------------

function ToastButton({ label, ...other }: ButtonProps & { label?: string }) {
  return (
    <Button variant="contained" color="inherit" {...other}>
      {label ?? 'default'}
    </Button>
  );
}
