import type { IconifyProps } from 'src/components/iconify';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/components/iconify';

export function KeyActionButton({
  title,
  icon,
  disabled,
  onClick,
}: {
  title: string;
  icon: IconifyProps['icon'];
  disabled?: boolean;
  onClick: () => void;
}) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton size="small" disabled={disabled} sx={actionButtonSx} onClick={onClick}>
          <Iconify icon={icon} width={14} />
        </IconButton>
      </span>
    </Tooltip>
  );
}

export function MetaDivider() {
  return (
    <Box component="span" sx={{ color: 'text.disabled' }}>
      |
    </Box>
  );
}

const actionButtonSx = { width: 28, height: 28 };
