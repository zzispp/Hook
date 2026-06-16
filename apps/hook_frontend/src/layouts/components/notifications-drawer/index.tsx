'use client';

import type { IconButtonProps } from '@mui/material/IconButton';
import type { NotificationItem as OperationsNotificationItem } from 'src/types/operations';

import { m } from 'framer-motion';
import { useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Badge from '@mui/material/Badge';
import Alert from '@mui/material/Alert';
import Drawer from '@mui/material/Drawer';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import CircularProgress from '@mui/material/CircularProgress';

import { Label } from 'src/components/label';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { varTap, varHover, transitionTap } from 'src/components/animate';

import { NotificationItem } from './notification-item';
import {
  type NotificationTab,
  type NotificationResource,
  useNotificationsDrawerState,
} from './notification-drawer-state';

type NotificationsDrawerState = ReturnType<typeof useNotificationsDrawerState>;

const TABS: { value: NotificationTab; labelKey: string; color: 'default' | 'info' | 'success' }[] =
  [
    { value: 'all', labelKey: 'operations.notifications.tabs.all', color: 'default' },
    { value: 'unread', labelKey: 'operations.notifications.tabs.unread', color: 'info' },
    { value: 'read', labelKey: 'operations.notifications.tabs.read', color: 'success' },
  ];

export type NotificationsDrawerProps = IconButtonProps;

export function NotificationsDrawer({ sx, ...other }: NotificationsDrawerProps) {
  const state = useNotificationsDrawerState();

  return (
    <>
      <NotificationBell
        sx={sx}
        label={state.t('operations.notifications.title')}
        unreadCount={state.resources.unread.total}
        onOpen={state.onOpen}
        {...other}
      />
      <Drawer
        open={state.open}
        onClose={state.onClose}
        anchor="right"
        slotProps={{
          backdrop: { invisible: true },
          paper: { sx: { width: 1, maxWidth: 420 } },
        }}
      >
        <NotificationHead state={state} />
        <NotificationTabs state={state} />
        <NotificationList
          resource={state.resources[state.currentTab]}
          t={state.t}
          onOpen={state.onOpenNotification}
          onDelete={state.onDeleteNotification}
        />
      </Drawer>
    </>
  );
}

function NotificationBell({
  label,
  onOpen,
  unreadCount,
  ...other
}: IconButtonProps & { label: string; unreadCount: number; onOpen: () => void }) {
  return (
    <IconButton
      component={m.button}
      whileTap={varTap(0.96)}
      whileHover={varHover(1.04)}
      transition={transitionTap()}
      aria-label={label}
      onClick={onOpen}
      {...other}
    >
      <Badge badgeContent={unreadCount} color="error">
        <Iconify width={24} icon="solar:bell-bing-bold-duotone" />
      </Badge>
    </IconButton>
  );
}

function NotificationHead({ state }: { state: NotificationsDrawerState }) {
  const totalUnread = state.resources.unread.total;
  const totalRead = state.resources.read.total;

  return (
    <Box sx={{ py: 2, pr: 1, pl: 2.5, minHeight: 68, display: 'flex', alignItems: 'center' }}>
      <Typography variant="h6" sx={{ flexGrow: 1 }}>
        {state.t('operations.notifications.title')}
      </Typography>
      {!!totalUnread && (
        <Tooltip title={state.t('operations.notifications.markAllRead')}>
          <span>
            <IconButton color="primary" disabled={state.busy} onClick={state.onMarkAllRead}>
              <Iconify icon="eva:done-all-fill" />
            </IconButton>
          </span>
        </Tooltip>
      )}
      {!!totalRead && (
        <Tooltip title={state.t('operations.notifications.deleteRead')}>
          <span>
            <IconButton
              color="error"
              disabled={state.busy}
              aria-label={state.t('operations.notifications.deleteRead')}
              onClick={state.onDeleteReadNotifications}
            >
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </span>
        </Tooltip>
      )}
      <IconButton onClick={state.onClose} sx={{ display: { xs: 'inline-flex', sm: 'none' } }}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Box>
  );
}

function NotificationTabs({ state }: { state: NotificationsDrawerState }) {
  const handleChangeTab = useCallback(
    (event: React.SyntheticEvent, value: string) => {
      state.setCurrentTab(value as NotificationTab);
    },
    [state]
  );

  return (
    <Tabs
      variant="fullWidth"
      value={state.currentTab}
      onChange={handleChangeTab}
      indicatorColor="custom"
    >
      {TABS.map((tab) => (
        <Tab
          key={tab.value}
          value={tab.value}
          iconPosition="end"
          label={state.t(tab.labelKey)}
          icon={
            <Label variant={tab.value === state.currentTab ? 'filled' : 'soft'} color={tab.color}>
              {state.resources[tab.value].total}
            </Label>
          }
        />
      ))}
    </Tabs>
  );
}

function NotificationList({
  t,
  resource,
  onOpen,
  onDelete,
}: {
  t: NotificationsDrawerState['t'];
  resource: NotificationResource;
  onOpen: (notification: OperationsNotificationItem) => void;
  onDelete: (notification: OperationsNotificationItem) => void;
}) {
  if (resource.error) {
    return (
      <Alert severity="error" sx={{ m: 2 }}>
        {resource.error.message}
      </Alert>
    );
  }

  if (resource.isLoading) {
    return (
      <Box sx={{ py: 6, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }

  if (!resource.items.length) {
    return (
      <Typography sx={{ p: 3 }} color="text.secondary">
        {t('operations.notifications.empty')}
      </Typography>
    );
  }

  return (
    <Scrollbar>
      <Box component="ul" sx={{ p: 0, m: 0, listStyle: 'none' }}>
        {resource.items.map((notification) => (
          <Box component="li" key={`${notification.source_type}:${notification.source_id}`}>
            <NotificationItem notification={notification} onOpen={onOpen} onDelete={onDelete} />
          </Box>
        ))}
      </Box>
    </Scrollbar>
  );
}
