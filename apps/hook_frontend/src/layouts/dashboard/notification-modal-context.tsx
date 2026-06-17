'use client';

import type { NotificationItem } from 'src/types/operations';
import type {
  ProviderQuickImportSyncEventDetailResponse,
} from 'src/types/provider-quick-import';

import { use, createContext } from 'react';

export type NotificationModalContextValue = {
  openFromNotification: (notification: NotificationItem) => Promise<void>;
  closeCurrentModal: () => Promise<void>;
  announcementQueue: string[];
  activeAnnouncementId?: string | null;
  activeQuickImportEvent?: ProviderQuickImportSyncEventDetailResponse | null;
  modalOpen: boolean;
  loading: boolean;
  error: Error | null;
};

export const NotificationModalContext = createContext<NotificationModalContextValue | undefined>(
  undefined
);

export function useNotificationModalContext() {
  const context = use(NotificationModalContext);

  if (!context) {
    throw new Error('useNotificationModalContext must be used inside NotificationModalProvider');
  }

  return context;
}
