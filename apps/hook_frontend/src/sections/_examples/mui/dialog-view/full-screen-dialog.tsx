import type { TransitionProps } from '@mui/material/transitions';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Slide from '@mui/material/Slide';
import Button from '@mui/material/Button';
import AppBar from '@mui/material/AppBar';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
import Toolbar from '@mui/material/Toolbar';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

type SlideTransitionProps = TransitionProps & {
  children: React.ReactElement;
  ref: React.RefObject<unknown>;
};

function Transition({ ref, ...other }: SlideTransitionProps) {
  return <Slide direction="up" ref={ref} {...other} />;
}

export function FullScreenDialog() {
  const openDialog = useBoolean();

  return (
    <>
      <Button variant="outlined" color="error" onClick={openDialog.onTrue}>
        Open full-screen dialog
      </Button>

      <Dialog
        fullScreen
        open={openDialog.value}
        onClose={openDialog.onFalse}
        slots={{ transition: Transition }}
      >
        <AppBar position="relative" color="default">
          <Toolbar>
            <IconButton color="inherit" edge="start" onClick={openDialog.onFalse}>
              <Iconify icon="mingcute:close-line" />
            </IconButton>

            <Typography variant="h6" sx={{ flex: 1, ml: 2 }}>
              Sound
            </Typography>

            <Button autoFocus color="inherit" variant="contained" onClick={openDialog.onFalse}>
              Save
            </Button>
          </Toolbar>
        </AppBar>

        <Box component="ul" sx={{ '& li': { display: 'flex' } }}>
          <li>
            <ListItemButton>
              <ListItemText primary="Phone ringtone" secondary="Titania" />
            </ListItemButton>
          </li>

          <Divider />

          <li>
            <ListItemButton>
              <ListItemText primary="Default notification ringtone" secondary="Tethys" />
            </ListItemButton>
          </li>
        </Box>
      </Dialog>
    </>
  );
}
