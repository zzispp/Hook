'use client';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

export function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <Stack spacing={2}>
      <Stack spacing={0.5}>
        <Typography variant="subtitle1">{title}</Typography>
        {description ? (
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            {description}
          </Typography>
        ) : null}
      </Stack>
      {children}
    </Stack>
  );
}
