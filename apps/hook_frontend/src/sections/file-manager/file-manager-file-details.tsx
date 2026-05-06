import type { DrawerProps } from '@mui/material/Drawer';
import type { IFile } from 'src/types/file';

import { useState, useCallback } from 'react';
import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Checkbox from '@mui/material/Checkbox';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { fData } from 'src/utils/format-number';
import { fDateTime } from 'src/utils/format-time';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { FileThumbnail, detectFileFormat } from 'src/components/file-thumbnail';

import { FileManagerShareDialog } from './file-manager-share-dialog';
import { FileManagerInvitedItem } from './file-manager-invited-item';

// ----------------------------------------------------------------------

type Props = DrawerProps & {
  file: IFile;
  favorited?: boolean;
  onClose: () => void;
  onDelete: () => void;
  onCopyLink: () => void;
  onFavorite?: () => void;
};

export function FileManagerFileDetails({
  file,
  open,
  onClose,
  onDelete,
  favorited,
  onFavorite,
  onCopyLink,
  ...other
}: Props) {
  const shareDialog = useBoolean();
  const showTags = useBoolean(true);
  const showProperties = useBoolean(true);

  const [inviteEmail, setInviteEmail] = useState('');
  const [tags, setTags] = useState(file?.tags.slice(0, 3));

  const hasShared = file?.shared && !!file?.shared.length;

  const handleChangeInvite = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setInviteEmail(event.target.value);
  }, []);

  const handleChangeTags = useCallback((newValue: string[]) => {
    setTags(newValue);
  }, []);

  const renderHead = () => (
    <Box
      sx={{
        p: 2.5,
        display: 'flex',
        alignItems: 'center',
      }}
    >
      <Typography variant="h6" sx={{ flexGrow: 1 }}>
        Info
      </Typography>

      <Checkbox
        color="warning"
        icon={<Iconify icon="eva:star-outline" />}
        checkedIcon={<Iconify icon="eva:star-fill" />}
        checked={favorited}
        onChange={onFavorite}
        slotProps={{
          input: {
            id: `favorite-details-${file.id}-checkbox`,
            'aria-label': `Favorite details ${file.id} checkbox`,
          },
        }}
      />
    </Box>
  );

  const renderProperties = () => {
    const fileDetails = [
      { label: 'Size', value: fData(file?.size) },
      { label: 'Modified', value: fDateTime(file?.modifiedAt) },
      { label: 'Type', value: detectFileFormat(file?.type) },
    ];

    return (
      <Stack spacing={1.5}>
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            typography: 'subtitle2',
            justifyContent: 'space-between',
          }}
        >
          Properties
          <IconButton size="small" onClick={showProperties.onToggle}>
            <Iconify
              icon={
                showProperties.value ? 'eva:arrow-ios-upward-fill' : 'eva:arrow-ios-downward-fill'
              }
            />
          </IconButton>
        </Box>

        {showProperties.value &&
          fileDetails.map((property) => (
            <Box
              key={property.label}
              sx={{ gap: 2, display: 'flex', typography: 'caption', textTransform: 'capitalize' }}
            >
              <Box component="span" sx={{ width: 80, color: 'text.secondary' }}>
                {property.label}
              </Box>
              {property.value}
            </Box>
          ))}
      </Stack>
    );
  };

  const renderTags = () => (
    <Stack spacing={1.5}>
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          typography: 'subtitle2',
          justifyContent: 'space-between',
        }}
      >
        Tags
        <IconButton size="small" onClick={showTags.onToggle}>
          <Iconify
            icon={showTags.value ? 'eva:arrow-ios-upward-fill' : 'eva:arrow-ios-downward-fill'}
          />
        </IconButton>
      </Box>

      {showTags.value && (
        <Autocomplete
          multiple
          freeSolo
          options={file?.tags.map((option) => option)}
          getOptionLabel={(option) => option}
          defaultValue={file?.tags.slice(0, 3)}
          value={tags}
          onChange={(event, newValue) => {
            handleChangeTags(newValue);
          }}
          renderInput={(params) => <TextField {...params} placeholder="#Add a tags" />}
          slotProps={{
            chip: { size: 'small', variant: 'soft' },
          }}
        />
      )}
    </Stack>
  );

  const renderShared = () => (
    <>
      <Box
        sx={{
          p: 2.5,
          display: 'flex',
          alignItems: 'center',
          typography: 'subtitle2',
          justifyContent: 'space-between',
        }}
      >
        Share with
        <IconButton
          size="small"
          color="primary"
          onClick={shareDialog.onTrue}
          sx={{
            width: 24,
            height: 24,
            bgcolor: 'primary.main',
            color: 'primary.contrastText',
            '&:hover': { bgcolor: 'primary.dark' },
          }}
        >
          <Iconify width={16} icon="mingcute:add-line" />
        </IconButton>
      </Box>

      {hasShared && (
        <Box component="ul" sx={{ pl: 2, pr: 1 }}>
          {file?.shared?.map((person) => (
            <FileManagerInvitedItem key={person.id} person={person} />
          ))}
        </Box>
      )}
    </>
  );

  return (
    <>
      <Drawer
        aria-hidden={!open}
        open={open}
        onClose={onClose}
        anchor="right"
        slotProps={{
          backdrop: { invisible: true },
          paper: { sx: { width: 320 } },
        }}
        {...other}
      >
        <Scrollbar>
          {renderHead()}

          <Stack
            spacing={2.5}
            sx={{ p: 2.5, justifyContent: 'center', bgcolor: 'background.neutral' }}
          >
            <FileThumbnail
              showImage
              file={file?.type === 'folder' ? file?.type : file?.url}
              sx={{ width: 'auto', height: 'auto', alignSelf: 'flex-start' }}
              slotProps={{
                img: { sx: { width: 320, height: 'auto', aspectRatio: '4/3', objectFit: 'cover' } },
                icon: { sx: { width: 64, height: 64 } },
              }}
            />

            <Typography variant="subtitle1" sx={{ wordBreak: 'break-all' }}>
              {file?.name}
            </Typography>

            <Divider sx={{ borderStyle: 'dashed' }} />

            {renderTags()}
            {renderProperties()}
          </Stack>

          {renderShared()}
        </Scrollbar>

        <Box sx={{ p: 2.5 }}>
          <Button
            fullWidth
            variant="soft"
            color="error"
            size="large"
            startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
            onClick={onDelete}
          >
            Delete
          </Button>
        </Box>
      </Drawer>

      <FileManagerShareDialog
        open={shareDialog.value}
        shared={file?.shared}
        inviteEmail={inviteEmail}
        onChangeInvite={handleChangeInvite}
        onCopyLink={onCopyLink}
        onClose={() => {
          shareDialog.onFalse();
          setInviteEmail('');
        }}
      />
    </>
  );
}
