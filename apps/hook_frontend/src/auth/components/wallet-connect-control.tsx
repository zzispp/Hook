'use client';

import type { IconifyProps } from 'src/components/iconify';

import { ConnectButton } from '@rainbow-me/rainbowkit';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { Iconify } from 'src/components/iconify';

type Props = {
  label: string;
  connectedActionLabel: string;
  loading: boolean;
  icon?: IconifyProps['icon'];
  onAction: VoidFunction;
};

export function WalletConnectControl({
  label,
  connectedActionLabel,
  loading,
  icon = 'solar:wad-of-money-bold',
  onAction,
}: Props) {
  return (
    <ConnectButton.Custom>
      {({ account, mounted, openAccountModal }) => {
        if (mounted && account) {
          return (
            <Box sx={{ gap: 1, display: 'grid', gridTemplateColumns: '1fr 1fr', width: 1 }}>
              <Button fullWidth color="inherit" variant="outlined" onClick={openAccountModal}>
                {account.displayName}
              </Button>
              <Button fullWidth color="inherit" variant="contained" loading={loading} onClick={onAction}>
                {connectedActionLabel}
              </Button>
            </Box>
          );
        }

        return (
          <Button
            fullWidth
            color="inherit"
            variant="outlined"
            loading={loading}
            startIcon={<Iconify icon={icon} />}
            onClick={onAction}
          >
            {label}
          </Button>
        );
      }}
    </ConnectButton.Custom>
  );
}
