'use client';

import type { Dispatch, ReactNode, SetStateAction } from 'react';
import type { NotificationItem } from 'src/types/operations';
import type { ProviderQuickImportSyncEventDetailResponse } from 'src/types/provider-quick-import';

import { useMemo, useState, useEffect, useCallback } from 'react';

import { getProviderQuickImportSyncEventDetail } from 'src/actions/provider-quick-import';
import { useAnnouncement, useUnreadAnnouncements, markNotificationSourceRead } from 'src/actions/operations';

import { AnnouncementContent } from 'src/sections/operations/announcement-content';

import { useAuthContext } from 'src/auth/hooks';

import { type ModalState } from './notification-modal-types';
import { NotificationModalDialog } from './notification-modal-dialog';
import {
  NotificationModalContext,
  type NotificationModalContextValue,
} from './notification-modal-context';

type Props = {
  children: ReactNode;
};

export function NotificationModalProvider({ children }: Props) {
  const { authenticated, loading: authLoading } = useAuthContext();
  const unreadAnnouncements = useUnreadAnnouncements(authenticated);
  const [state, setState] = useState<ModalState>({ kind: 'closed' });
  const [queue, setQueue] = useState<string[]>([]);
  const [event, setEvent] = useState<ProviderQuickImportSyncEventDetailResponse | null>(null);
  const [eventLoading, setEventLoading] = useState(false);
  const [modalError, setModalError] = useState<Error | null>(null);

  const announcementId = state.kind === 'announcement' ? state.id : '';
  const announcement = useAnnouncement(announcementId);
  const announcementContent = announcement.data ? <AnnouncementContent announcement={announcement.data} /> : null;

  useEffect(() => {
    syncAnnouncementQueue({
      authLoading,
      authenticated,
      isLoading: unreadAnnouncements.isLoading,
      error: unreadAnnouncements.error,
      items: unreadAnnouncements.items,
      state,
      setQueue,
      setState,
      setModalError,
    });
  }, [
    authLoading,
    authenticated,
    state,
    unreadAnnouncements.error,
    unreadAnnouncements.isLoading,
    unreadAnnouncements.items,
  ]);

  const openFromNotification = useCallback(
    async (notification: NotificationItem) => {
      if (notification.source_type === 'announcement') {
        await openAnnouncementNotification(notification, unreadAnnouncements.refresh, setState);
        return;
      }

      if (notification.source_type === 'provider_quick_import_sync') {
        await openQuickImportNotification(notification, setEvent, setState, setEventLoading, setModalError);
      }
    },
    [unreadAnnouncements.refresh]
  );

  const closeCurrentModal = useCallback(async () => {
    if (state.kind === 'announcement') {
      await closeAnnouncementModal(state.id, unreadAnnouncements.refresh, setQueue, setState);
      return;
    }

    setEvent(null);
    setState(queue[0] ? { kind: 'announcement', id: queue[0] } : { kind: 'closed' });
  }, [queue, state, unreadAnnouncements.refresh]);

  const contextValue = useMemo<NotificationModalContextValue>(
    () => ({
      openFromNotification,
      closeCurrentModal,
      announcementQueue: queue,
      activeAnnouncementId: announcementId || null,
      activeQuickImportEvent: event,
      modalOpen: state.kind !== 'closed',
      loading: eventLoading || (state.kind === 'announcement' && !!announcementId && announcement.isLoading),
      error: modalError ?? announcement.error ?? unreadAnnouncements.error ?? null,
    }),
    [
      announcement.error,
      announcement.isLoading,
      announcementId,
      closeCurrentModal,
      event,
      eventLoading,
      modalError,
      openFromNotification,
      queue,
      state.kind,
      unreadAnnouncements.error,
    ]
  );

  return (
    <NotificationModalContext value={contextValue}>
      {children}
      <NotificationModalDialog
        state={state}
        announcementContent={announcementContent}
        quickImportEvent={event}
        loading={contextValue.loading}
        error={contextValue.error}
        onClose={closeCurrentModal}
      />
    </NotificationModalContext>
  );
}

function syncAnnouncementQueue({
  authLoading,
  authenticated,
  isLoading,
  error,
  items,
  state,
  setQueue,
  setState,
  setModalError,
}: {
  authLoading: boolean;
  authenticated: boolean;
  isLoading: boolean;
  error: Error | undefined;
  items: Array<{ id: string }>;
  state: ModalState;
  setQueue: Dispatch<SetStateAction<string[]>>;
  setState: Dispatch<SetStateAction<ModalState>>;
  setModalError: Dispatch<SetStateAction<Error | null>>;
}) {
  if (authLoading || !authenticated || isLoading) {
    return;
  }

  if (error) {
    setModalError(error);
    return;
  }

  const ids = items.map((item) => item.id);
  setQueue(ids);
  setState((current) => {
    if (state.kind !== 'closed') {
      return current;
    }
    return ids[0] ? { kind: 'announcement', id: ids[0] } : current;
  });
}

async function openAnnouncementNotification(
  notification: NotificationItem,
  refreshUnread: () => Promise<unknown>,
  setState: Dispatch<SetStateAction<ModalState>>
) {
  if (notification.is_unread) {
    await markNotificationSourceRead('announcement', notification.source_id);
    await refreshUnread();
  }

  setState({ kind: 'announcement', id: notification.source_id });
}

async function openQuickImportNotification(
  notification: NotificationItem,
  setEvent: Dispatch<SetStateAction<ProviderQuickImportSyncEventDetailResponse | null>>,
  setState: Dispatch<SetStateAction<ModalState>>,
  setLoading: Dispatch<SetStateAction<boolean>>,
  setModalError: Dispatch<SetStateAction<Error | null>>
) {
  if (notification.is_unread) {
    await markNotificationSourceRead(notification.source_type, notification.source_id);
  }

  setLoading(true);
  setModalError(null);
  try {
    const detail = await getProviderQuickImportSyncEventDetail(notification.source_id);
    setEvent(detail);
    setState({ kind: 'quickImport', id: notification.source_id });
  } catch (error) {
    setModalError(error instanceof Error ? error : new Error('Request failed'));
    throw error;
  } finally {
    setLoading(false);
  }
}

async function closeAnnouncementModal(
  announcementId: string,
  refreshUnread: () => Promise<unknown>,
  setQueue: Dispatch<SetStateAction<string[]>>,
  setState: Dispatch<SetStateAction<ModalState>>
) {
  await markNotificationSourceRead('announcement', announcementId);
  await refreshUnread();
  setQueue((current) => {
    const remaining = current.filter((id) => id !== announcementId);
    setState(remaining[0] ? { kind: 'announcement', id: remaining[0] } : { kind: 'closed' });
    return remaining;
  });
}
