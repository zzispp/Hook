import type { IDateValue } from 'src/types/common';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Avatar from '@mui/material/Avatar';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { fDate } from 'src/utils/format-time';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

type Props = {
  name: string;
  message: string;
  tagUser?: string;
  avatarUrl: string;
  hasReply?: boolean;
  postedAt: IDateValue;
};

export function PostCommentItem({ name, avatarUrl, message, tagUser, postedAt, hasReply }: Props) {
  const reply = useBoolean();

  return (
    <Box
      sx={{
        pt: 3,
        gap: 2,
        display: 'flex',
        position: 'relative',
        ...(hasReply && { pl: 8 }),
      }}
    >
      <Avatar alt={name} src={avatarUrl} sx={{ width: 48, height: 48 }} />

      <Box
        sx={[
          (theme) => ({
            pb: 3,
            display: 'flex',
            flex: '1 1 auto',
            flexDirection: 'column',
            borderBottom: `solid 1px ${theme.vars.palette.divider}`,
          }),
        ]}
      >
        <Typography variant="subtitle2" sx={{ mb: 0.5 }}>
          {name}
        </Typography>

        <Typography variant="caption" sx={{ color: 'text.disabled' }}>
          {fDate(postedAt)}
        </Typography>

        <Typography variant="body2" sx={{ mt: 1 }}>
          {tagUser && (
            <Box component="strong" sx={{ mr: 0.5 }}>
              @{tagUser}
            </Box>
          )}
          {message}
        </Typography>

        {reply.value && (
          <TextField fullWidth autoFocus placeholder="Write comment..." sx={{ mt: 2 }} />
        )}
      </Box>

      {!hasReply && (
        <Button
          size="small"
          color={reply.value ? 'primary' : 'inherit'}
          startIcon={<Iconify icon="solar:pen-bold" width={16} />}
          onClick={reply.onToggle}
          sx={{ right: 0, position: 'absolute' }}
        >
          Reply
        </Button>
      )}
    </Box>
  );
}
