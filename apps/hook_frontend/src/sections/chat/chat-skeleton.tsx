import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Skeleton from '@mui/material/Skeleton';
import CircularProgress from '@mui/material/CircularProgress';

// ----------------------------------------------------------------------

type ChatMessageSkeletonProps = BoxProps & {
  itemCount?: number;
};

export function ChatNavItemSkeleton({ sx, itemCount = 6, ...other }: ChatMessageSkeletonProps) {
  return Array.from({ length: itemCount }, (_, index) => (
    <Box
      key={index}
      sx={[
        {
          gap: 2,
          px: 2.5,
          py: 1.5,
          display: 'flex',
          alignItems: 'center',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Skeleton variant="circular" sx={{ width: 48, height: 48 }} />

      <Box sx={{ flex: '1 1 auto' }}>
        <Skeleton sx={{ mb: 1, width: 0.75, height: 10 }} />
        <Skeleton sx={{ width: 0.5, height: 10 }} />
      </Box>
    </Box>
  ));
}

// ----------------------------------------------------------------------

export function ChatHeaderSkeleton({ sx, ...other }: BoxProps) {
  return (
    <Box
      sx={[
        {
          width: 1,
          display: 'flex',
          alignItems: 'center',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Skeleton variant="circular" sx={{ width: 40, height: 40 }} />

      <Box sx={{ mx: 2, flex: '1 1 auto' }}>
        <Skeleton sx={{ mb: 1, width: 96, height: 10 }} />
        <Skeleton sx={{ width: 40, height: 10 }} />
      </Box>

      <Skeleton variant="circular" sx={{ width: 28, height: 28 }} />
      <Skeleton variant="circular" sx={{ width: 28, height: 28, mx: 1 }} />
      <Skeleton variant="circular" sx={{ width: 28, height: 28, mr: 1 }} />
    </Box>
  );
}

// ----------------------------------------------------------------------

export function ChatRoomSkeleton({ sx, ...other }: BoxProps) {
  return (
    <Box
      sx={[
        {
          pt: 5,
          flexGrow: 1,
          display: 'flex',
          alignItems: 'center',
          flexDirection: 'column',
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Skeleton variant="circular" sx={{ width: 96, height: 96 }} />
      <Skeleton sx={{ mb: 1, mt: 2, height: 10, width: 0.65 }} />
      <Skeleton sx={{ mb: 5, width: 0.35, height: 10 }} />
      <CircularProgress color="inherit" thickness={2} />
    </Box>
  );
}
