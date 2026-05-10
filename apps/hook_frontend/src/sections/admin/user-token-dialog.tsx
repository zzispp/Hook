'use client';

import type { SystemUser } from 'src/types/rbac';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { defaultGroupCode } from '../api-tokens/api-token-management-utils';
import { TokenManagementPanel, useTokenManagementPanelState } from '../api-tokens/token-management-panel';

type Props = {
  user: SystemUser | null;
  onClose: VoidFunction;
};

export function UserTokenDialog({ user, onClose }: Props) {
  const { t } = useTranslate('admin');
  const panel = useTokenManagementPanelState({
    scope: 'admin',
    fixedUserId: user?.id,
    disabled: !user,
  });

  return (
    <Dialog fullWidth maxWidth="xl" open={Boolean(user)} onClose={onClose}>
      <DialogTitle>
        <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
          <Stack spacing={0.5}>
            <Typography variant="h6">{t('userTokens.title')}</Typography>
            <Typography variant="caption" color="text.secondary">
              {user ? `${user.username} · ${user.email}` : ''}
            </Typography>
          </Stack>
          <Stack direction="row" spacing={1}>
            <Button
              variant="contained"
              startIcon={<Iconify icon="mingcute:add-line" />}
              onClick={() => panel.dialog.openCreate(defaultGroupCode(panel.groups.items))}
            >
              {t('actions.addApiToken')}
            </Button>
            <IconButton onClick={onClose}>
              <Iconify icon="solar:close-circle-bold" />
            </IconButton>
          </Stack>
        </Stack>
      </DialogTitle>
      <DialogContent sx={{ pb: 2 }}>
        <TokenManagementPanel state={panel} />
      </DialogContent>
    </Dialog>
  );
}
