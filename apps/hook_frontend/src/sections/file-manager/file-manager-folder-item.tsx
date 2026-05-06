import type { IFolderManager } from 'src/types/file';
import type { FileItemProps } from './file-manager-file-item-slots';

import { useState, useCallback } from 'react';
import { useBoolean, usePopover, useCopyToClipboard } from 'minimal-shared/hooks';

import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';

import { fData } from 'src/utils/format-number';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { ConfirmDialog } from 'src/components/custom-dialog';
import { CustomPopover } from 'src/components/custom-popover';

import { FileManagerShareDialog } from './file-manager-share-dialog';
import { FileManagerFileDetails } from './file-manager-file-details';
import { FileManagerCreateFolderDialog } from './file-manager-create-folder-dialog';
import {
  FileItem,
  FileItemIcon,
  FileItemInfo,
  FileItemAvatar,
  FileItemActions,
  FileItemActionOverlay,
} from './file-manager-file-item-slots';

// ----------------------------------------------------------------------

type Props = FileItemProps & {
  selected?: boolean;
  onDelete: () => void;
  onSelect?: () => void;
  folder: IFolderManager;
};

export function FileManagerFolderItem({
  sx,
  folder,
  selected,
  onSelect,
  onDelete,
  ...other
}: Props) {
  const shareDialog = useBoolean();
  const confirmDialog = useBoolean();
  const detailsDrawer = useBoolean();
  const editFolderDialog = useBoolean();

  const checkbox = useBoolean();
  const favorite = useBoolean(folder.isFavorited);

  const menuActions = usePopover();

  const { copy } = useCopyToClipboard();

  const [inviteEmail, setInviteEmail] = useState('');
  const [folderName, setFolderName] = useState(folder.name);

  const handleChangeInvite = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setInviteEmail(event.target.value);
  }, []);

  const handleChangeFolderName = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setFolderName(event.target.value);
  }, []);

  const handleCopy = useCallback(() => {
    toast.success('Copied!');
    copy(folder.url);
  }, [copy, folder.url]);

  const renderMenuActions = () => (
    <CustomPopover
      open={menuActions.open}
      anchorEl={menuActions.anchorEl}
      onClose={menuActions.onClose}
      slotProps={{ arrow: { placement: 'right-top' } }}
    >
      <MenuList>
        <MenuItem
          onClick={() => {
            menuActions.onClose();
            handleCopy();
          }}
        >
          <Iconify icon="eva:link-2-fill" />
          Copy Link
        </MenuItem>

        <MenuItem
          onClick={() => {
            menuActions.onClose();
            shareDialog.onTrue();
          }}
        >
          <Iconify icon="solar:share-bold" />
          Share
        </MenuItem>

        <MenuItem
          onClick={() => {
            menuActions.onClose();
            editFolderDialog.onTrue();
          }}
        >
          <Iconify icon="solar:pen-bold" />
          Edit
        </MenuItem>

        <Divider sx={{ borderStyle: 'dashed' }} />

        <MenuItem
          onClick={() => {
            confirmDialog.onTrue();
            menuActions.onClose();
          }}
          sx={{ color: 'error.main' }}
        >
          <Iconify icon="solar:trash-bin-trash-bold" />
          Delete
        </MenuItem>
      </MenuList>
    </CustomPopover>
  );

  const renderShareDialog = () => (
    <FileManagerShareDialog
      open={shareDialog.value}
      shared={folder.shared}
      inviteEmail={inviteEmail}
      onChangeInvite={handleChangeInvite}
      onCopyLink={handleCopy}
      onClose={() => {
        shareDialog.onFalse();
        setInviteEmail('');
      }}
    />
  );

  const renderConfirmDialog = () => (
    <ConfirmDialog
      open={confirmDialog.value}
      onClose={confirmDialog.onFalse}
      title="Delete"
      content="Are you sure want to delete?"
      action={
        <Button variant="contained" color="error" onClick={onDelete}>
          Delete
        </Button>
      }
    />
  );

  const renderEditFolderDialog = () => (
    <FileManagerCreateFolderDialog
      open={editFolderDialog.value}
      onClose={editFolderDialog.onFalse}
      title="Edit Folder"
      onUpdate={() => {
        editFolderDialog.onFalse();
        setFolderName(folderName);
        console.info('UPDATE FOLDER', folderName);
      }}
      folderName={folderName}
      onChangeFolderName={handleChangeFolderName}
    />
  );

  const renderFileDetailsDrawer = () => (
    <FileManagerFileDetails
      file={folder}
      favorited={favorite.value}
      onFavorite={favorite.onToggle}
      onCopyLink={handleCopy}
      open={detailsDrawer.value}
      onClose={detailsDrawer.onFalse}
      onDelete={() => {
        detailsDrawer.onFalse();
        onDelete();
      }}
    />
  );

  return (
    <>
      <FileItem variant="outlined" selected={selected} sx={sx} {...other}>
        <FileItemActionOverlay onClick={detailsDrawer.onTrue} />

        <FileItemIcon
          id={folder.id}
          onMouseEnter={checkbox.onTrue}
          onMouseLeave={checkbox.onFalse}
          hovered={checkbox.value}
          checked={selected}
          onChange={onSelect}
        />

        <FileItemInfo
          type="folder"
          title={folder.name}
          values={[fData(folder.size), `${folder.totalFiles} files`]}
        />

        <FileItemAvatar sharedUsers={folder.shared} />

        <FileItemActions
          id={folder.id}
          checked={favorite.value}
          onChange={favorite.onToggle}
          openMenu={menuActions.open}
          onOpenMenu={menuActions.onOpen}
        />
      </FileItem>

      {renderMenuActions()}

      {renderShareDialog()}
      {renderConfirmDialog()}
      {renderEditFolderDialog()}

      {renderFileDetailsDrawer()}
    </>
  );
}
