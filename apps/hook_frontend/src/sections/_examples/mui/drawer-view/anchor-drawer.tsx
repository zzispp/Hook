import { Fragment, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

type Anchor = 'top' | 'left' | 'bottom' | 'right';
const anchorKeys: Anchor[] = ['left', 'right', 'top', 'bottom'];

export function AnchorDrawer() {
  const [state, setState] = useState<Record<Anchor, boolean>>({
    top: false,
    left: false,
    bottom: false,
    right: false,
  });

  const toggleDrawer = useCallback(
    (anchor: Anchor, open: boolean) => (event: React.KeyboardEvent | React.MouseEvent) => {
      if (
        event.type === 'keydown' &&
        ['Tab', 'Shift'].includes((event as React.KeyboardEvent).key)
      ) {
        return;
      }
      setState((prev) => ({ ...prev, [anchor]: open }));
    },
    []
  );

  const renderListItem = (text: string) => (
    <ListItem key={text} disablePadding>
      <ListItemButton>
        <ListItemIcon>
          <Iconify icon="solar:file-corrupted-bold-duotone" />
        </ListItemIcon>
        <ListItemText primary={text} />
      </ListItemButton>
    </ListItem>
  );

  const renderList = (anchor: Anchor) => (
    <Box
      sx={{ width: anchor === 'top' || anchor === 'bottom' ? 'auto' : 250 }}
      role="presentation"
      onClick={toggleDrawer(anchor, false)}
      onKeyDown={toggleDrawer(anchor, false)}
    >
      <List>{['Inbox', 'Starred', 'Send email', 'Drafts'].map(renderListItem)}</List>
      <Divider />
      <List>{['All mail', 'Trash', 'Spam'].map(renderListItem)}</List>
    </Box>
  );

  return (
    <>
      {anchorKeys.map((anchor) => (
        <Fragment key={anchor}>
          <Button
            variant="outlined"
            onClick={toggleDrawer(anchor, true)}
            sx={{ textTransform: 'capitalize' }}
          >
            {anchor}
          </Button>

          <Drawer anchor={anchor} open={state[anchor]} onClose={toggleDrawer(anchor, false)}>
            {renderList(anchor)}
          </Drawer>
        </Fragment>
      ))}
    </>
  );
}
