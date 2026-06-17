'use client';

import type { Provider, ProviderApiKey } from 'src/types/provider';

import { useState, useCallback } from 'react';

type KeyTarget = {
  provider: Provider;
  apiKey: ProviderApiKey;
};

export function useProviderQuickImportActionState() {
  const [appendProvider, setAppendProvider] = useState<Provider | null>(null);
  const [bindProvider, setBindProvider] = useState<Provider | null>(null);
  const [resolutionTarget, setResolutionTarget] = useState<KeyTarget | null>(null);
  const [keyMappingsTarget, setKeyMappingsTarget] = useState<KeyTarget | null>(null);

  const openResolution = useCallback((provider: Provider, apiKey: ProviderApiKey) => {
    setResolutionTarget({ provider, apiKey });
  }, []);

  const openKeyMappings = useCallback((provider: Provider, apiKey: ProviderApiKey) => {
    setKeyMappingsTarget({ provider, apiKey });
  }, []);

  return {
    appendProvider,
    bindProvider,
    keyMappingsTarget,
    resolutionTarget,
    openAppend: setAppendProvider,
    closeAppend: () => setAppendProvider(null),
    openBind: setBindProvider,
    closeBind: () => setBindProvider(null),
    openResolution,
    closeResolution: () => setResolutionTarget(null),
    openKeyMappings,
    closeKeyMappings: () => setKeyMappingsTarget(null),
  };
}
