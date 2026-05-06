import type { IFileManager } from 'src/types/file';
import type { FileItemProps } from './file-manager-file-item-slots';

import { useState, useCallback } from 'react';
import { useBoolean, usePopover, useCopyToClipboard } from 'minimal-shared/hooks';

import Divider from '@mui/material/Divider';
import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';

import { fData } from 'src/utils/format-number';
import { fDateTime } from 'src/utils/format-time';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { FileThumbnail } from 'src/components/file-thumbnail';
import { CustomPopover } from 'src/components/custom-popover';

import { FileManagerShareDialog } from './file-manager-share-dialog';
import { FileManagerFileDetails } from './file-manager-file-details';
import {
  FileItem,
  FileItemInfo,
  FileItemAvatar,
  FileItemActions,
  FileItemActionOverlay,
} from './file-manager-file-item-slots';

// ----------------------------------------------------------------------

type Props = FileItemProps & {
  file: IFileManager;
  onDelete: () => void;
};

export function FileRecentItem({ file, onDelete, sx, ...other }: Props) {
  const { copy } = useCopyToClipboard();

  const menuActions = usePopover();

  const shareDialog = useBoolean();
  const detailsDrawer = useBoolean();
  const favorite = useBoolean(file.isFavorited);

  const [inviteEmail, setInviteEmail] = useState('');

  const handleChangeInvite = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setInviteEmail(event.target.value);
  }, []);

  const handleCopy = useCallback(() => {
    toast.success('Copied!');
    copy(file.url);
  }, [copy, file.url]);

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
          Copy link
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

        <Divider sx={{ borderStyle: 'dashed' }} />

        <MenuItem
          onClick={() => {
            menuActions.onClose();
            onDelete();
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
      shared={file.shared}
      inviteEmail={inviteEmail}
      onChangeInvite={handleChangeInvite}
      onCopyLink={handleCopy}
      onClose={() => {
        shareDialog.onFalse();
        setInviteEmail('');
      }}
    />
  );

  const renderFileDetailsDrawer = () => (
    <FileManagerFileDetails
      file={file}
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
      <FileItem
        variant="outlined"
        sx={[
          {
            p: { xs: 2.5, sm: 2 },
            alignItems: { xs: 'unset', sm: 'center' },
            flexDirection: { xs: 'column', sm: 'row' },
          },
          ...(Array.isArray(sx) ? sx : [sx]),
        ]}
        {...other}
      >
        <FileItemActionOverlay onClick={detailsDrawer.onTrue} />

        <FileThumbnail file={file.type} />

        <FileItemInfo
          type="recent-file"
          title={file.name}
          values={[fData(file.size), fDateTime(file.modifiedAt)]}
        />

        <FileItemAvatar sharedUsers={file.shared} />

        <FileItemActions
          id={file.id}
          checked={favorite.value}
          onChange={favorite.onToggle}
          openMenu={menuActions.open}
          onOpenMenu={menuActions.onOpen}
          sx={{ position: { xs: 'absolute', sm: 'unset' } }}
        />
      </FileItem>

      {renderMenuActions()}
      {renderFileDetailsDrawer()}
      {renderShareDialog()}
    </>
  );
}
