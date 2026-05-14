'use client';

import type { Announcement } from 'src/types/operations';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { fToNow } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';
import { DashboardContent } from 'src/layouts/dashboard';
import { useAnnouncements } from 'src/actions/operations';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';
import { useDashboardBreadcrumbs } from 'src/layouts/dashboard/use-dashboard-breadcrumbs';

import { Iconify } from 'src/components/iconify';
import { CustomBreadcrumbs } from 'src/components/custom-breadcrumbs';

import { AnnouncementTypeLabel } from './operation-labels';

export function AnnouncementListView() {
  const { t } = useTranslate('admin');
  const breadcrumbs = useDashboardBreadcrumbs({ headingCode: DASHBOARD_MENU_CODES.announcements });
  const [search, setSearch] = useState('');
  const announcements = useAnnouncements(0, 50, { search });

  const handleSearch = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setSearch(event.target.value);
  }, []);

  return (
    <DashboardContent maxWidth="lg">
      <CustomBreadcrumbs heading={breadcrumbs.heading} links={breadcrumbs.links} sx={{ mb: 4 }} />
      <TextField
        fullWidth
        value={search}
        onChange={handleSearch}
        placeholder={t('operations.announcement.searchPlaceholder')}
        sx={{ mb: 3 }}
      />
      <Stack spacing={2}>
        {announcements.items.map((item) => (
          <AnnouncementCard key={item.id} item={item} />
        ))}
        {!announcements.isLoading && !announcements.items.length ? (
          <Typography color="text.secondary">{t('operations.announcement.empty')}</Typography>
        ) : null}
      </Stack>
    </DashboardContent>
  );
}

function AnnouncementCard({ item }: { item: Announcement }) {
  const { t } = useTranslate('admin');

  return (
    <Card sx={{ p: 3 }}>
      <Stack spacing={1.5}>
        <Stack direction="row" spacing={1} sx={{ alignItems: 'center', flexWrap: 'wrap' }}>
          <AnnouncementTypeLabel value={item.announcement_type} />
          {item.pinned ? <Iconify icon="solar:flag-bold" color="warning.main" /> : null}
          <Typography variant="caption" color="text.disabled">
            {fToNow(item.updated_at)}
          </Typography>
        </Stack>
        <Typography variant="h6">{item.title}</Typography>
        <Typography
          color="text.secondary"
          sx={{
            display: '-webkit-box',
            overflow: 'hidden',
            WebkitLineClamp: 2,
            WebkitBoxOrient: 'vertical',
          }}
        >
          {item.content_markdown}
        </Typography>
        <Box>
          <Button
            component={RouterLink}
            href={`${paths.dashboard.announcements}/${item.id}`}
            endIcon={<Iconify icon="eva:diagonal-arrow-right-up-fill" />}
          >
            {t('common.details')}
          </Button>
        </Box>
      </Stack>
    </Card>
  );
}
