import type { DialogProps } from '@mui/material/Dialog';

import { useBoolean } from 'minimal-shared/hooks';
import { useRef, useState, useEffect, useCallback } from 'react';

import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';

// ----------------------------------------------------------------------

const LOREM_TEXT = `
Cras mattis consectetur purus sit amet fermentum. Cras justo odio, dapibus ac facilisis in,
egestas eget quam. Morbi leo risus, porta ac consectetur ac, vestibulum at eros. Praesent commodo
cursus magna, vel scelerisque nisl consectetur et.
`;

// ----------------------------------------------------------------------

export function ScrollDialog() {
  const openDialog = useBoolean();

  const descriptionElementRef = useRef<HTMLElement>(null);

  const [scroll, setScroll] = useState<DialogProps['scroll']>('paper');

  const handleClickOpen = useCallback(
    (scrollType: DialogProps['scroll']) => () => {
      openDialog.onTrue();
      setScroll(scrollType);
    },
    [openDialog, setScroll]
  );

  useEffect(() => {
    if (openDialog.value) {
      const { current: descriptionElement } = descriptionElementRef;
      if (descriptionElement) {
        descriptionElement.focus();
      }
    }
  }, [openDialog.value]);

  return (
    <>
      <Button variant="outlined" onClick={handleClickOpen('paper')} sx={{ mr: 2 }}>
        Scroll=paper
      </Button>

      <Button variant="outlined" onClick={handleClickOpen('body')}>
        Scroll=body
      </Button>

      <Dialog open={openDialog.value} onClose={openDialog.onFalse} scroll={scroll}>
        <DialogTitle sx={{ pb: 2 }}>Subscribe</DialogTitle>

        <DialogContent dividers={scroll === 'paper'}>
          <DialogContentText ref={descriptionElementRef} tabIndex={-1}>
            {[...new Array(50)].map(() => LOREM_TEXT).join('\n')}
          </DialogContentText>
        </DialogContent>

        <DialogActions>
          <Button onClick={openDialog.onFalse}>Cancel</Button>

          <Button variant="contained" onClick={openDialog.onFalse}>
            Subscribe
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
}
