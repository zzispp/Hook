import type { PriorityItem } from './provider-priority-utils';
import type { SystemSettingsUpdate } from 'src/types/system-setting';
import type { Provider, ProviderSchedulingMode } from 'src/types/provider';

import { useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales/use-locales';
import { updateSystemSettings } from 'src/actions/system-settings';

import { toast } from 'src/components/snackbar';

import {
  orderProviders,
  parsePriorities,
  movePriorityItem,
  changeItemPriority,
  savePriorityChanges,
} from './provider-priority-utils';

export type ProviderPriorityDialogProps = {
  open: boolean;
  providers: Provider[];
  loading: boolean;
  schedulingMode: ProviderSchedulingMode;
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
    const priorities = parsePriorities(form.items);
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
      await savePriorityState({ ...props, ...form, cacheTtlMinutes, priorities });
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
  schedulingMode,
  cacheAffinityTtlMinutes: initialCacheAffinityTtlMinutes,
}: ProviderPriorityDialogProps) {
  const [items, setItems] = useState(orderProviders(providers));
  const [mode, setMode] = useState<ProviderSchedulingMode>(schedulingMode);
  const [cacheAffinityTtlMinutes, setCacheAffinityTtlMinutes] = useState(
    String(initialCacheAffinityTtlMinutes || DEFAULT_CACHE_AFFINITY_TTL_MINUTES)
  );
  const [editingId, setEditingId] = useState<string | null>(null);
  const [draggingId, setDraggingId] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setItems(orderProviders(providers));
    setMode(schedulingMode);
    setCacheAffinityTtlMinutes(String(initialCacheAffinityTtlMinutes || DEFAULT_CACHE_AFFINITY_TTL_MINUTES));
    setEditingId(null);
    setDraggingId(null);
  }, [open, providers, schedulingMode, initialCacheAffinityTtlMinutes]);

  const changePriority = useCallback((id: string, value: string) => {
    setItems((current) => changeItemPriority(current, id, value));
  }, []);

  const dropOn = useDropHandler({ draggingId, setDraggingId, setItems });

  return {
    cacheAffinityTtlMinutes,
    changePriority,
    draggingId,
    dropOn,
    editingId,
    items,
    mode,
    setCacheAffinityTtlMinutes,
    setDraggingId,
    setEditingId,
    setMode,
  };
}

function useDropHandler({
  draggingId,
  setDraggingId,
  setItems,
}: {
  draggingId: string | null;
  setDraggingId: (value: string | null) => void;
  setItems: React.Dispatch<React.SetStateAction<PriorityItem[]>>;
}) {
  return useCallback(
    (targetId: string) => {
      if (!draggingId || draggingId === targetId) return;
      setItems((current) => movePriorityItem(current, draggingId, targetId));
      setDraggingId(null);
    },
    [draggingId, setDraggingId, setItems]
  );
}

async function savePriorityState({
  cacheTtlMinutes,
  items,
  mode,
  providers,
  priorities,
  schedulingMode,
}: {
  cacheTtlMinutes: number | null;
  items: PriorityItem[];
  mode: ProviderSchedulingMode;
  providers: Provider[];
  priorities: Map<string, number>;
  schedulingMode: ProviderSchedulingMode;
}) {
  await savePriorityChanges(items, providers, priorities);
  const patch = settingsPatch(mode, schedulingMode, cacheTtlMinutes);
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
  cacheTtlMinutes: number | null
): SystemSettingsUpdate | null {
  if (mode === 'cache_affinity') {
    return {
      scheduling_mode: mode,
      cache_affinity_ttl_minutes: cacheTtlMinutes!,
    };
  }
  return mode === schedulingMode ? null : { scheduling_mode: mode };
}
