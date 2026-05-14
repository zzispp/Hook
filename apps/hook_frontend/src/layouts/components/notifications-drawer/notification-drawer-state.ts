'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { NotificationItem as OperationsNotificationItem } from 'src/types/operations';

import { useBoolean } from 'minimal-shared/hooks';
import { useMemo, useState, useCallback } from 'react';

import { useRouter } from 'src/routes/hooks';

import { useTranslate } from 'src/locales/use-locales';
import {
  useNotifications,
  deleteNotification,
  markNotificationRead,
  markAllNotificationsRead,
} from 'src/actions/operations';

import { toast } from 'src/components/snackbar';

export type NotificationTab = 'all' | 'unread' | 'read';
export type NotificationResource = ReturnType<typeof useNotifications>;
export type NotificationResources = Record<NotificationTab, NotificationResource>;

export function useNotificationsDrawerState() {
  const { t } = useTranslate('admin');
  const router = useRouter();
  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();
  const [currentTab, setCurrentTab] = useState<NotificationTab>('all');
  const [busy, setBusy] = useState(false);
  const resources = useNotificationResources();
  const openDrawer = useOpenDrawerAction(onOpen, resources);
  const changeTab = useNotificationTabAction(setCurrentTab, resources);
  const actions = useNotificationActions({ t, router, onClose, setBusy });

  return {
    t,
    open,
    busy,
    onOpen: openDrawer,
    onClose,
    resources,
    currentTab,
    setCurrentTab: changeTab,
    ...actions,
  };
}

function useNotificationResources(): NotificationResources {
  const all = useNotifications();
  const unread = useNotifications('unread');
  const read = useNotifications('read');

  return useMemo(() => ({ all, unread, read }), [all, read, unread]);
}

function useOpenDrawerAction(onOpen: () => void, resources: NotificationResources) {
  return useCallback(() => {
    onOpen();
    void refreshNotificationResources(resources);
  }, [onOpen, resources]);
}

function useNotificationTabAction(
  setCurrentTab: Dispatch<SetStateAction<NotificationTab>>,
  resources: NotificationResources
) {
  return useCallback(
    (tab: NotificationTab) => {
      setCurrentTab(tab);
      void resources[tab].refresh();
    },
    [resources, setCurrentTab]
  );
}

async function refreshNotificationResources(resources: NotificationResources) {
  await Promise.all([resources.all.refresh(), resources.unread.refresh(), resources.read.refresh()]);
}

function useNotificationActions({
  t,
  router,
  onClose,
  setBusy,
}: {
  t: ReturnType<typeof useTranslate>['t'];
  router: ReturnType<typeof useRouter>;
  onClose: () => void;
  setBusy: Dispatch<SetStateAction<boolean>>;
}) {
  const onMarkAllRead = useMarkAllReadAction(t, setBusy);
  const onOpenNotification = useOpenNotificationAction(t, router, onClose);
  const onDeleteNotification = useDeleteNotificationAction(t);

  return { onMarkAllRead, onOpenNotification, onDeleteNotification };
}

function useMarkAllReadAction(
  t: ReturnType<typeof useTranslate>['t'],
  setBusy: Dispatch<SetStateAction<boolean>>
) {
  return useCallback(async () => {
    setBusy(true);
    try {
      await markAllNotificationsRead();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setBusy(false);
    }
  }, [setBusy, t]);
}

function useOpenNotificationAction(
  t: ReturnType<typeof useTranslate>['t'],
  router: ReturnType<typeof useRouter>,
  onClose: () => void
) {
  return useCallback(
    async (item: OperationsNotificationItem) => {
      try {
        if (item.is_unread) {
          await markNotificationRead(item);
        }
        onClose();
        router.push(item.link_path);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      }
    },
    [onClose, router, t]
  );
}

function useDeleteNotificationAction(t: ReturnType<typeof useTranslate>['t']) {
  return useCallback(
    async (item: OperationsNotificationItem) => {
      try {
        await deleteNotification(item);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
      }
    },
    [t]
  );
}
