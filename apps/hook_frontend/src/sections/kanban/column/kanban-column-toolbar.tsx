import type { BoxProps } from '@mui/material/Box';
import type { UseColumnDndReturn } from '../hooks/use-column-dnd';

import { varAlpha } from 'minimal-shared/utils';
import { useBoolean, usePopover } from 'minimal-shared/hooks';
import { useId, useRef, useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { CustomPopover } from 'src/components/custom-popover';

import { KanbanInputName } from '../components/kanban-input-name';

// ----------------------------------------------------------------------

type Props = BoxProps & {
  totalTasks?: number;
  columnName: string;
  dragHandleRef?: UseColumnDndReturn['dragHandleRef'];
  onClearColumn?: () => void;
  onDeleteColumn?: () => void;
  onToggleAddTask?: () => void;
  onUpdateColumn?: (inputName: string) => void;
};

export function KanbanColumnToolBar({
  sx,
  dragHandleRef,
  columnName,
  totalTasks,
  onClearColumn,
  onDeleteColumn,
  onUpdateColumn,
  onToggleAddTask,
  ...other
}: Props) {
  const uniqueId = useId();

  const renameRef = useRef<HTMLInputElement>(null);

  const menuActions = usePopover();
  const confirmDialog = useBoolean();

  const [name, setName] = useState(columnName);
  const [isRenaming, setIsRenaming] = useState(false);

  useEffect(() => {
    if (isRenaming && !menuActions.open && renameRef.current) {
      renameRef.current.focus();
    }
  }, [isRenaming, menuActions.open]);

  const handleChangeName = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setName(event.target.value);
  }, []);

  const handleKeyUpUpdateColumn = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === 'Enter') {
        renameRef.current?.blur();
        onUpdateColumn?.(name);
      }
    },
    [name, onUpdateColumn]
  );

  const handleRename = useCallback(() => {
    setIsRenaming(true);
    menuActions.onClose();
  }, [menuActions]);

  const handleClear = useCallback(() => {
    onClearColumn?.();
    menuActions.onClose();
  }, [menuActions, onClearColumn]);

  const handleDelete = useCallback(() => {
    confirmDialog.onTrue();
    menuActions.onClose();
  }, [confirmDialog, menuActions]);

  const renderMenuActions = () => (
    <CustomPopover
      open={menuActions.open}
      anchorEl={menuActions.anchorEl}
      onClose={menuActions.onClose}
    >
      <MenuList>
        <MenuItem onClick={handleRename}>
          <Iconify icon="solar:pen-bold" />
          Rename
        </MenuItem>

        <MenuItem onClick={handleClear}>
          <Iconify icon="solar:eraser-bold" />
          Clear
        </MenuItem>

        <MenuItem onClick={handleDelete} sx={{ color: 'error.main' }}>
          <Iconify icon="solar:trash-bin-trash-bold" />
          Delete
        </MenuItem>
      </MenuList>
    </CustomPopover>
  );

  const renderConfirmDialog = () => (
    <ConfirmDialog
      open={confirmDialog.value}
      onClose={confirmDialog.onFalse}
      title="Delete"
      content={
        <>
          Are you sure you want to delete this column?
          <Box sx={{ typography: 'caption', color: 'error.main', mt: 2 }}>
            <strong>NOTE:</strong> All tasks in this column will also be deleted.
          </Box>
        </>
      }
      action={
        <Button
          variant="contained"
          color="error"
          onClick={() => {
            onDeleteColumn?.();
            confirmDialog.onFalse();
          }}
        >
          Delete
        </Button>
      }
    />
  );

  const renderDragHandle = () => (
    <Box
      ref={dragHandleRef}
      component="span"
      sx={{
        top: 0,
        left: 0,
        width: 1,
        height: 1,
        cursor: 'grab',
        position: 'absolute',
      }}
    />
  );

  return (
    <>
      <Box
        sx={[
          {
            display: 'flex',
            alignItems: 'center',
            position: 'relative',
            pt: 'var(--kanban-column-pt)',
            px: 'var(--kanban-column-px)',
          },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...other}
      >
        {renderDragHandle()}

        <Label
          sx={[
            (theme) => ({
              borderRadius: '50%',
              borderColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.24),
            }),
          ]}
        >
          {totalTasks}
        </Label>

        <KanbanInputName
          inputRef={renameRef}
          placeholder="Column name"
          value={name}
          onChange={handleChangeName}
          onKeyUp={handleKeyUpUpdateColumn}
          onFocus={() => setIsRenaming(true)}
          onBlur={() => setIsRenaming(false)}
          inputProps={{ id: `${columnName}-${uniqueId}-column-input` }}
          sx={{ mx: 1 }}
        />

        <IconButton size="small" color="inherit" onClick={onToggleAddTask}>
          <Iconify icon="solar:add-circle-bold" />
        </IconButton>

        <IconButton
          size="small"
          color={menuActions.open ? 'inherit' : 'default'}
          onClick={menuActions.onOpen}
        >
          <Iconify icon="solar:menu-dots-bold-duotone" />
        </IconButton>

        <IconButton size="small" sx={{ pointerEvents: 'none' }}>
          <Iconify icon="custom:drag-dots-fill" />
        </IconButton>
      </Box>

      {renderMenuActions()}
      {renderConfirmDialog()}
    </>
  );
}
