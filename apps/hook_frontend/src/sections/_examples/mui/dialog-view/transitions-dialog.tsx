import type { TransitionProps } from '@mui/material/transitions';

import { useBoolean } from 'minimal-shared/hooks';

import Slide from '@mui/material/Slide';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

// ----------------------------------------------------------------------

type SlideTransitionProps = TransitionProps & {
  children: React.ReactElement;
  ref: React.RefObject<unknown>;
};

function Transition({ ref, ...other }: SlideTransitionProps) {
  return <Slide direction="up" ref={ref} {...other} />;
}

export function TransitionsDialog() {
  const openDialog = useBoolean();

  return (
    <div>
      <Button variant="outlined" color="success" onClick={openDialog.onTrue}>
        Transitions dialog
      </Button>

      <Dialog
        keepMounted
        open={openDialog.value}
        onClose={openDialog.onFalse}
        slots={{ transition: Transition }}
      >
        <DialogTitle>{`Use Google's location service?`}</DialogTitle>

        <DialogContent sx={{ color: 'text.secondary' }}>
          Let Google help apps determine location. This means sending anonymous location data to
          Google, even when no apps are running.
        </DialogContent>

        <DialogActions>
          <Button variant="outlined" onClick={openDialog.onFalse}>
            Disagree
          </Button>
          <Button variant="contained" onClick={openDialog.onFalse} autoFocus>
            Agree
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}
