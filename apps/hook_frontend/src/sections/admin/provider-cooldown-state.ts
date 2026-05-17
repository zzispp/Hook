'use client';

import type { useTranslate } from 'src/locales/use-locales';

import { useState, useCallback } from 'react';

import { useProviderCooldowns, releaseProviderCooldown } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';

import {
  toProviderCooldownFilters,
  DEFAULT_PROVIDER_COOLDOWN_FILTERS,
} from './provider-cooldown-table';

type CooldownTable = {
  page: number;
  rowsPerPage: number;
  onResetPage: () => void;
};

export function useProviderCooldownState(
  t: ReturnType<typeof useTranslate>['t'],
  table: CooldownTable
) {
  const [filters, setFilters] = useState(DEFAULT_PROVIDER_COOLDOWN_FILTERS);
  const [releasingId, setReleasingId] = useState<string | null>(null);
  const cooldowns = useProviderCooldowns(
    table.page,
    table.rowsPerPage,
    toProviderCooldownFilters(filters)
  );

  const changeFilters = useCallback(
    (nextFilters: typeof DEFAULT_PROVIDER_COOLDOWN_FILTERS) => {
      table.onResetPage();
      setFilters(nextFilters);
    },
    [table]
  );

  const release = useCallback(
    async (providerId: string) => {
      setReleasingId(providerId);
      try {
        await releaseProviderCooldown(providerId);
        toast.success(t('messages.providerCooldownReleased'));
        await cooldowns.refresh();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      } finally {
        setReleasingId(null);
      }
    },
    [cooldowns, t]
  );

  return {
    cooldowns,
    filters,
    releasingId,
    changeFilters,
    release,
  };
}
