'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';

import { useState, useCallback } from 'react';

type KeyTarget = {
  provider: Provider;
  apiKey: ProviderApiKey;
};

export function useProviderQuickImportActionState() {
  const [appendProvider, setAppendProvider] = useState<Provider | null>(null);
  const [resolutionTarget, setResolutionTarget] = useState<KeyTarget | null>(null);
  const [modelAssociationsTarget, setModelAssociationsTarget] = useState<KeyTarget | null>(null);

  const openResolution = useCallback((provider: Provider, apiKey: ProviderApiKey) => {
    setResolutionTarget({ provider, apiKey });
  }, []);

  const openModelAssociations = useCallback((provider: Provider, apiKey: ProviderApiKey) => {
    setModelAssociationsTarget({ provider, apiKey });
  }, []);

  return {
    appendProvider,
    modelAssociationsTarget,
    resolutionTarget,
    openAppend: setAppendProvider,
    closeAppend: () => setAppendProvider(null),
    openResolution,
    closeResolution: () => setResolutionTarget(null),
    openModelAssociations,
    closeModelAssociations: () => setModelAssociationsTarget(null),
  };
}
