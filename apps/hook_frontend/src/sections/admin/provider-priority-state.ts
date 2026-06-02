import type { SystemSettingsUpdate } from 'src/types/system-setting';
import type { PriorityItem, PriorityKind, PriorityItemsByFormat } from './provider-priority-utils';
import type { Provider, ProviderApiKey, ProviderPriorityMode, ProviderSchedulingMode } from 'src/types/provider';

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
  schedulingMode: ProviderSchedulingMode;
  priorityMode: ProviderPriorityMode;
  keyPrioritySnapshotInitialized: boolean;
  cacheAffinityTtlMinutes: number;
  onClose: () => void;
  onSaved: () => void;
};

const DEFAULT_CACHE_AFFINITY_TTL_MINUTES = 5;

export type PriorityDialogState = ReturnType<typeof usePriorityDialogState>;

export function usePriorityDialogState(props: ProviderPriorityDialogProps) {
  const { t } = useTranslate('admin');
  const form = usePriorityFormState(props);
  const [submitting, setSubmitting] = useState(false);

  const save = useCallback(async () => {
    const saveItems = prioritySaveItems(form.kind, form.items, form.itemsByFormat);
    const priorities = parsePriorities(saveItems);
    const cacheTtlMinutes = parseCacheTtlMinutes(form.cacheAffinityTtlMinutes);
    if (!priorities) {
      toast.error(t('messages.providerPriorityInvalid'));
      return;
    }
    if (form.mode === 'cache_affinity' && cacheTtlMinutes === null) {
      toast.error(t('messages.providerCacheAffinityTtlInvalid'));
      return;
    }

    setSubmitting(true);
    try {
      await savePriorityState({ ...props, ...form, items: saveItems, cacheTtlMinutes, priorities });
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
  schedulingMode,
  priorityMode,
  keyPrioritySnapshotInitialized,
  cacheAffinityTtlMinutes: initialCacheAffinityTtlMinutes,
}: ProviderPriorityDialogProps) {
  const [kind, setKind] = useState<PriorityKind>(priorityMode);
  const [items, setItems] = useState(priorityItems(priorityMode, providers, keysByProvider, keyPrioritySnapshotInitialized));
  const [itemsByFormat, setItemsByFormat] = useState(keyPriorityItemsByFormat(providers, keysByProvider, keyPrioritySnapshotInitialized));
  const [activeFormat, setActiveFormat] = useState(firstPriorityFormat(keysByProvider));
  const [mode, setMode] = useState<ProviderSchedulingMode>(schedulingMode);
  const [cacheAffinityTtlMinutes, setCacheAffinityTtlMinutes] = useState(
    String(initialCacheAffinityTtlMinutes || DEFAULT_CACHE_AFFINITY_TTL_MINUTES)
  );
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setKind(priorityMode);
    setItems(priorityItems(priorityMode, providers, keysByProvider, keyPrioritySnapshotInitialized));
    setItemsByFormat(keyPriorityItemsByFormat(providers, keysByProvider, keyPrioritySnapshotInitialized));
    setActiveFormat(firstPriorityFormat(keysByProvider));
    setMode(schedulingMode);
    setCacheAffinityTtlMinutes(String(initialCacheAffinityTtlMinutes || DEFAULT_CACHE_AFFINITY_TTL_MINUTES));
    setEditingId(null);
    setDraggingId(null);
  }, [open, providers, keysByProvider, priorityMode, keyPrioritySnapshotInitialized, schedulingMode, initialCacheAffinityTtlMinutes]);

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
    cacheAffinityTtlMinutes,
    changeKind,
    changePriority,
    activeFormat,
    draggingId,
    dropOn,
    editingId,
    items,
    itemsByFormat,
    kind,
    mode,
    priorityFormats: priorityFormats(keysByProvider),
    setActiveFormat: changeActiveFormat,
    setCacheAffinityTtlMinutes,
    setDraggingId,
    setEditingId,
    setMode,
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
  cacheTtlMinutes,
  items,
  kind,
  keysByProvider,
  mode,
  providers,
  priorities,
  priorityMode,
  keyPrioritySnapshotInitialized,
  schedulingMode,
}: {
  cacheTtlMinutes: number | null;
  items: PriorityItem[];
  kind: PriorityKind;
  keysByProvider: Record<string, ProviderApiKey[]>;
  mode: ProviderSchedulingMode;
  providers: Provider[];
  priorities: Map<string, number>;
  priorityMode: ProviderPriorityMode;
  keyPrioritySnapshotInitialized: boolean;
  schedulingMode: ProviderSchedulingMode;
}) {
  await savePriorityChanges(kind, items, providers, keysByProvider, priorities);
  const patch = settingsPatch(mode, schedulingMode, cacheTtlMinutes, kind, priorityMode, keyPrioritySnapshotInitialized);
  if (patch) await updateSystemSettings(patch);
}

function parseCacheTtlMinutes(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return DEFAULT_CACHE_AFFINITY_TTL_MINUTES;
  const number = Number(trimmed);
  return Number.isInteger(number) && number > 0 ? number : null;
}

function settingsPatch(
  mode: ProviderSchedulingMode,
  schedulingMode: ProviderSchedulingMode,
  cacheTtlMinutes: number | null,
  priorityKind: PriorityKind,
  priorityMode: ProviderPriorityMode,
  keyPrioritySnapshotInitialized: boolean
): SystemSettingsUpdate | null {
  const priorityPatch: Partial<SystemSettingsUpdate> = priorityKind === priorityMode ? {} : { provider_priority_mode: priorityKind };
  if (priorityKind === 'key' && !keyPrioritySnapshotInitialized) {
    priorityPatch.key_priority_snapshot_initialized = true;
  }
  if (mode === 'cache_affinity') {
    return {
      scheduling_mode: mode,
      cache_affinity_ttl_minutes: cacheTtlMinutes!,
      ...priorityPatch,
    };
  }
  if (mode !== schedulingMode) return { scheduling_mode: mode, ...priorityPatch };
  return Object.keys(priorityPatch).length > 0 ? priorityPatch : null;
}
