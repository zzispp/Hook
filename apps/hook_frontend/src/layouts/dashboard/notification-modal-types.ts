'use client';

import type { ReactNode } from 'react';
import type { ProviderQuickImportSyncEventDetailResponse } from 'src/types/provider-quick-import';

export type ModalState =
  | { kind: 'closed' }
  | { kind: 'announcement'; id: string }
  | { kind: 'quickImport'; id: string };

export type NotificationModalDialogProps = {
  state: ModalState;
  announcementContent?: ReactNode;
  quickImportEvent: ProviderQuickImportSyncEventDetailResponse | null;
  loading: boolean;
  error: Error | null;
  onClose: () => Promise<void>;
};
