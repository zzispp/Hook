import type { Provider, ProviderApiKey } from 'src/types/provider';
import type { PriorityItem, PriorityKind, PriorityItemsByFormat } from './provider-priority-utils';

import { useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import {
  orderKeys,
  orderProviders,
  parsePriorities,
  priorityFormats,
  movePriorityItem,
  orderKeysByFormat,
  changeItemPriority,
  savePriorityChanges,
} from './provider-priority-utils';

export type ProviderPriorityDialogProps = {
  open: boolean;
  providers: Provider[];
  keysByProvider: Record<string, ProviderApiKey[]>;
  loading: boolean;
  initialKind: PriorityKind;
  keyPrioritySnapshotInitialized: boolean;
  onClose: () => void;
  onSaved: () => void;
};

export type PriorityDialogState = ReturnType<typeof usePriorityDialogState>;

export function usePriorityDialogState(props: ProviderPriorityDialogProps) {
  const { t } = useTranslate('admin');
  const form = usePriorityFormState(props);
  const [submitting, setSubmitting] = useState(false);

  const save = useCallback(async () => {
    const saveItems = prioritySaveItems(form.kind, form.items, form.itemsByFormat);
    const priorities = parsePriorities(saveItems);
    if (!priorities) {
      toast.error(t('messages.providerPriorityInvalid'));
      return;
    }

    setSubmitting(true);
    try {
      await savePriorityState({ ...props, ...form, items: saveItems, priorities });
      toast.success(t('messages.providerPriorityUpdated'));
      props.onSaved();
      props.onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [form, props, t]);

  return { ...form, save, submitting };
}

function usePriorityFormState({
  open,
  providers,
  keysByProvider,
  initialKind,
  keyPrioritySnapshotInitialized,
}: ProviderPriorityDialogProps) {
  const [kind, setKind] = useState<PriorityKind>(initialKind);
  const [items, setItems] = useState(priorityItems(initialKind, providers, keysByProvider, keyPrioritySnapshotInitialized));
  const [itemsByFormat, setItemsByFormat] = useState(keyPriorityItemsByFormat(providers, keysByProvider, keyPrioritySnapshotInitialized));
  const [activeFormat, setActiveFormat] = useState(firstPriorityFormat(keysByProvider));
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setKind(initialKind);
    setItems(priorityItems(initialKind, providers, keysByProvider, keyPrioritySnapshotInitialized));
    setItemsByFormat(keyPriorityItemsByFormat(providers, keysByProvider, keyPrioritySnapshotInitialized));
    setActiveFormat(firstPriorityFormat(keysByProvider));
    setEditingId(null);
    setDraggingId(null);
  }, [open, providers, keysByProvider, initialKind, keyPrioritySnapshotInitialized]);

  const changeKind = useCallback(
    (nextKind: PriorityKind) => {
      setKind(nextKind);
      setItems(priorityItems(nextKind, providers, keysByProvider, keyPrioritySnapshotInitialized));
      setItemsByFormat(keyPriorityItemsByFormat(providers, keysByProvider, keyPrioritySnapshotInitialized));
      setActiveFormat((current) => current || firstPriorityFormat(keysByProvider));
      setEditingId(null);
      setDraggingId(null);
    },
    [keyPrioritySnapshotInitialized, keysByProvider, providers]
  );

  const changePriority = useCallback(
    (id: string, value: string) => {
      if (kind === 'key') {
        setItemsByFormat((current) => changeFormatItemPriority(current, activeFormat, id, value));
        return;
      }
      setItems((current) => changeItemPriority(current, id, value));
    },
    [activeFormat, kind]
  );

  const changeActiveFormat = useCallback((value: string) => {
    setActiveFormat(value);
    setEditingId(null);
    setDraggingId(null);
  }, []);

  const dropOn = useDropHandler({
    activeFormat,
    draggingId,
    kind,
    setDraggingId,
    setItems,
    setItemsByFormat,
  });

  return {
    changeKind,
    changePriority,
    activeFormat,
    draggingId,
    dropOn,
    editingId,
    items,
    itemsByFormat,
    kind,
    priorityFormats: priorityFormats(keysByProvider),
    setActiveFormat: changeActiveFormat,
    setDraggingId,
    setEditingId,
  };
}

function priorityItems(
  kind: PriorityKind,
  providers: Provider[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  keyPrioritySnapshotInitialized: boolean
) {
  const keyPrioritySource = keyPrioritySnapshotInitialized ? 'global' : 'internal';
  const firstFormat = firstPriorityFormat(keysByProvider);
  return kind === 'key' && firstFormat ? orderKeys(providers, keysByProvider, keyPrioritySource, firstFormat) : orderProviders(providers);
}

function keyPriorityItemsByFormat(
  providers: Provider[],
  keysByProvider: Record<string, ProviderApiKey[]>,
  keyPrioritySnapshotInitialized: boolean
) {
  const keyPrioritySource = keyPrioritySnapshotInitialized ? 'global' : 'internal';
  return orderKeysByFormat(providers, keysByProvider, keyPrioritySource);
}

function firstPriorityFormat(keysByProvider: Record<string, ProviderApiKey[]>) {
  return priorityFormats(keysByProvider)[0] ?? '';
}

function useDropHandler({
  activeFormat,
  draggingId,
  kind,
  setDraggingId,
  setItems,
  setItemsByFormat,
}: {
  activeFormat: string;
  draggingId: string | null;
  kind: PriorityKind;
  setDraggingId: (value: string | null) => void;
  setItems: React.Dispatch<React.SetStateAction<PriorityItem[]>>;
  setItemsByFormat: React.Dispatch<React.SetStateAction<PriorityItemsByFormat>>;
}) {
  return useCallback(
    (targetId: string) => {
      if (!draggingId || draggingId === targetId) return;
      if (kind === 'key') {
        setItemsByFormat((current) => moveFormatPriorityItem(current, activeFormat, draggingId, targetId));
        setDraggingId(null);
        return;
      }
      setItems((current) => movePriorityItem(current, draggingId, targetId));
      setDraggingId(null);
    },
    [activeFormat, draggingId, kind, setDraggingId, setItems, setItemsByFormat]
  );
}

function changeFormatItemPriority(
  itemsByFormat: PriorityItemsByFormat,
  apiFormat: string,
  id: string,
  value: string
) {
  return {
    ...itemsByFormat,
    [apiFormat]: changeItemPriority(itemsByFormat[apiFormat] ?? [], id, value),
  };
}

function moveFormatPriorityItem(
  itemsByFormat: PriorityItemsByFormat,
  apiFormat: string,
  sourceId: string,
  targetId: string
) {
  return {
    ...itemsByFormat,
    [apiFormat]: movePriorityItem(itemsByFormat[apiFormat] ?? [], sourceId, targetId),
  };
}

function prioritySaveItems(
  kind: PriorityKind,
  items: PriorityItem[],
  itemsByFormat: PriorityItemsByFormat
) {
  return kind === 'key' ? Object.values(itemsByFormat).flat() : items;
}

async function savePriorityState({
  items,
  kind,
  keysByProvider,
  providers,
  priorities,
  keyPrioritySnapshotInitialized,
}: {
  items: PriorityItem[];
  kind: PriorityKind;
  keysByProvider: Record<string, ProviderApiKey[]>;
  providers: Provider[];
  priorities: Map<string, number>;
  keyPrioritySnapshotInitialized: boolean;
}) {
  await savePriorityChanges(kind, items, providers, keysByProvider, priorities);
  if (kind === 'key' && !keyPrioritySnapshotInitialized) {
    await updateSystemSettings({ key_priority_snapshot_initialized: true });
  }
}
