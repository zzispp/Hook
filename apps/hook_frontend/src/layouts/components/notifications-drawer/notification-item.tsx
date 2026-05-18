'use client';

import type { NotificationItem as OperationsNotificationItem } from 'src/types/operations';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import ListItemText from '@mui/material/ListItemText';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import ListItemButton from '@mui/material/ListItemButton';

import { fToNow } from 'src/utils/format-time';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

export type NotificationItemProps = {
  notification: OperationsNotificationItem;
  onOpen: (notification: OperationsNotificationItem) => void;
  onDelete: (notification: OperationsNotificationItem) => void;
};

export function NotificationItem({ notification, onOpen, onDelete }: NotificationItemProps) {
  const { t, currentLang } = useTranslate('admin');

  return (
    <ListItemButton
      disableRipple
      onClick={() => onOpen(notification)}
      sx={[
        (theme) => ({
          p: 2.5,
          gap: 1,
          alignItems: 'flex-start',
          position: 'relative',
          borderBottom: `dashed 1px ${theme.vars.palette.divider}`,
        }),
      ]}
    >
      {notification.is_unread ? <UnreadBadge /> : null}
      <ListItemAvatar>
        <Box
          sx={{
            width: 40,
            height: 40,
            display: 'flex',
            borderRadius: '50%',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: 'background.neutral',
          }}
        >
          <Iconify width={24} icon={sourceIcon(notification.source_type)} />
        </Box>
      </ListItemAvatar>
      <ListItemText
        primary={notification.title}
        secondary={
          <>
            {fToNow(notification.created_at, currentLang.adapterLocale)}
            <Box
              component="span"
              sx={{ width: 2, height: 2, borderRadius: '50%', bgcolor: 'currentColor' }}
            />
            {categoryLabel(notification.category, t)}
          </>
        }
        slotProps={{
          primary: { sx: { mb: 0.5, typography: 'subtitle2' } },
          secondary: {
            sx: {
              gap: 0.5,
              display: 'flex',
              alignItems: 'center',
              typography: 'caption',
              color: 'text.disabled',
            },
          },
        }}
      />
      <Tooltip title={t('common.delete')}>
        <IconButton
          size="small"
          color="error"
          onClick={(event) => {
            event.stopPropagation();
            onDelete(notification);
          }}
        >
          <Iconify icon="solar:trash-bin-trash-bold" />
        </IconButton>
      </Tooltip>
    </ListItemButton>
  );
}

function UnreadBadge() {
  return (
    <Box
      sx={{
        top: 26,
        width: 8,
        height: 8,
        right: 20,
        borderRadius: '50%',
        bgcolor: 'info.main',
        position: 'absolute',
      }}
    />
  );
}

function sourceIcon(sourceType: string) {
  return sourceType === 'ticket'
    ? 'solar:chat-round-dots-bold'
    : 'solar:bell-bing-bold-duotone';
}

function categoryLabel(category: string, t: ReturnType<typeof useTranslate>['t']) {
  if (category.startsWith('announcement.')) {
    const type = category.replace('announcement.', '');
    return `${t('operations.notifications.categories.announcement')} / ${t(
      `operations.announcement.types.${type}`
    )}`;
  }

  if (category === 'ticket') {
    return t('operations.notifications.categories.ticket');
  }

  return category;
}
