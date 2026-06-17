'use client';

import type { Announcement } from 'src/types/operations';

import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { fDateTime } from 'src/utils/format-time';

import { Markdown } from 'src/components/markdown';

import { AnnouncementTypeLabel } from './operation-labels';

export function AnnouncementContent({ announcement }: { announcement: Announcement }) {
  return (
    <Stack spacing={3}>
      <Stack spacing={1}>
        <AnnouncementTypeLabel value={announcement.announcement_type} />
        <Typography variant="h4">{announcement.title}</Typography>
        <Typography variant="caption" color="text.disabled">
          {fDateTime(announcement.updated_at)}
        </Typography>
      </Stack>
      <Markdown>{announcement.content_markdown}</Markdown>
    </Stack>
  );
}
