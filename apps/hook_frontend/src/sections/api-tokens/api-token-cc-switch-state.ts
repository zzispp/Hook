'use client';

import type { ApiToken } from 'src/types/api-token';
import type { GlobalModelResponse } from 'src/types/model';
import type { TokenScope } from './api-token-management-types';
import type { CcSwitchFormat } from './api-token-cc-switch-utils';
import type { DisplayApiEndpoint } from './api-token-endpoints-utils';

import { useMemo, useState, useEffect } from 'react';

import { loadCcSwitchTokenContext } from './api-token-cc-switch-loader';
import {
  CC_SWITCH_FORMATS,
  tokenModelOptions,
  buildCcSwitchDeepLink,
  modelsForCcSwitchFormat,
} from './api-token-cc-switch-utils';

type CcSwitchStep = 'endpoint' | 'format' | 'model';

type UseCcSwitchImportDialogArgs = {
  scope: TokenScope;
  t: (key: string, options?: Record<string, unknown>) => string;
  catalog: GlobalModelResponse[];
  apiEndpoints: DisplayApiEndpoint[];
  siteName?: string | null;
};

export type CcSwitchImportDialogState = ReturnType<typeof useCcSwitchImportDialog>;

export function useCcSwitchImportDialog({
  scope,
  t,
  catalog,
  apiEndpoints,
  siteName,
}: UseCcSwitchImportDialogArgs) {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState<CcSwitchStep>('endpoint');
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState(false);
  const [targetToken, setTargetToken] = useState<ApiToken | null>(null);
  const [rawToken, setRawToken] = useState('');
  const [modelIds, setModelIds] = useState<string[]>([]);
  const [selectedEndpointId, setSelectedEndpointId] = useState('');
  const [selectedFormat, setSelectedFormat] = useState<CcSwitchFormat>('claude');
  const [selectedModelId, setSelectedModelId] = useState('');
  const [error, setError] = useState<string | null>(null);

  const selectedEndpoint = useMemo(
    () => apiEndpoints.find((endpoint) => endpoint.id === selectedEndpointId) ?? null,
    [apiEndpoints, selectedEndpointId]
  );
  const modelOptions = useMemo(
    () => tokenModelOptions(modelIds, catalog),
    [catalog, modelIds]
  );
  const selectedFormatModels = useMemo(
    () => modelsForCcSwitchFormat(modelOptions, selectedFormat),
    [modelOptions, selectedFormat]
  );
  const availableFormats = useMemo(
    () => CC_SWITCH_FORMATS.filter((format) => modelsForCcSwitchFormat(modelOptions, format).length > 0),
    [modelOptions]
  );

  useEffect(() => {
    if (!open) return;
    syncSelectedEndpoint({ apiEndpoints, selectedEndpointId, setSelectedEndpointId });
  }, [apiEndpoints, open, selectedEndpointId]);

  useEffect(() => {
    if (!open) return;
    syncSelectedFormat({
      availableFormats,
      selectedFormat,
      setSelectedFormat,
    });
  }, [availableFormats, open, selectedFormat]);

  useEffect(() => {
    if (!open) return;
    syncSelectedModel({
      selectedFormatModels,
      selectedModelId,
      setSelectedModelId,
    });
  }, [open, selectedFormatModels, selectedModelId]);

  function openImport(token: ApiToken) {
    resetDialog();
    setOpen(true);
    setTargetToken(token);
    setSelectedEndpointId(apiEndpoints[0]?.id ?? '');
  }

  function closeDialog() {
    resetDialog();
  }

  function resetDialog() {
    setOpen(false);
    setStep('endpoint');
    setLoading(false);
    setImporting(false);
    setTargetToken(null);
    setRawToken('');
    setModelIds([]);
    setSelectedEndpointId('');
    setSelectedFormat('claude');
    setSelectedModelId('');
    setError(null);
  }

  function backToEndpoint() {
    setStep('endpoint');
    setModelIds([]);
    setRawToken('');
    setError(null);
  }

  function backToFormat() {
    setStep('format');
  }

  async function goToFormatStep() {
    if (!targetToken || !selectedEndpoint) {
      setError(t('tokens.ccSwitch.noApiEndpoints'));
      return;
    }
    await loadTokenModels(targetToken.id, selectedEndpoint.url);
  }

  function goToModelStep() {
    if (selectedFormatModels.length === 0) {
      setError(t('tokens.ccSwitch.noModelsForFormat'));
      return;
    }
    setError(null);
    setStep('model');
  }

  async function reload() {
    if (!targetToken || !selectedEndpoint) return;
    await loadTokenModels(targetToken.id, selectedEndpoint.url);
  }

  function importToCcSwitch() {
    if (!targetToken || !selectedEndpoint || !rawToken || !selectedModelId) {
      setError(t('tokens.ccSwitch.incompleteSelection'));
      return;
    }

    setImporting(true);
    try {
      const deeplink = buildCcSwitchDeepLink({
        format: selectedFormat,
        modelId: selectedModelId,
        rawToken,
        tokenName: targetToken.name,
        baseUrl: selectedEndpoint.url,
        siteName,
        homepage: window.location.origin,
      });
      window.open(deeplink, '_self');
      closeDialog();
    } finally {
      setImporting(false);
    }
  }

  return {
    availableFormats,
    apiEndpoints,
    backToEndpoint,
    backToFormat,
    canContinueToFormat: !!selectedEndpoint && !loading,
    canContinueToModel: selectedFormatModels.length > 0 && !loading,
    canImport: !!selectedEndpoint && !!rawToken && !!selectedModelId && !loading && !importing,
    closeDialog,
    error,
    goToFormatStep,
    goToModelStep,
    importToCcSwitch,
    importing,
    loading,
    modelOptions,
    open,
    openImport,
    reload,
    selectedEndpoint,
    selectedEndpointId,
    selectedFormat,
    selectedFormatModels,
    selectedModelId,
    setSelectedEndpointId,
    setSelectedFormat,
    setSelectedModelId,
    step,
    targetToken,
  };

  async function loadTokenModels(tokenId: string, baseUrl: string) {
    setLoading(true);
    setError(null);
    setRawToken('');
    setModelIds([]);
    try {
      const context = await loadCcSwitchTokenContext({ scope, tokenId, baseUrl });
      setRawToken(context.rawToken);
      setModelIds(context.modelIds);
      setStep('format');
      if (context.modelIds.length === 0) {
        setError(t('tokens.ccSwitch.noAvailableModels'));
      }
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : t('tokens.ccSwitch.loadFailed'));
    } finally {
      setLoading(false);
    }
  }
}

function syncSelectedEndpoint({
  apiEndpoints,
  selectedEndpointId,
  setSelectedEndpointId,
}: {
  apiEndpoints: DisplayApiEndpoint[];
  selectedEndpointId: string;
  setSelectedEndpointId: (endpointId: string) => void;
}) {
  if (apiEndpoints.some((endpoint) => endpoint.id === selectedEndpointId)) {
    return;
  }
  setSelectedEndpointId(apiEndpoints[0]?.id ?? '');
}

function syncSelectedFormat({
  availableFormats,
  selectedFormat,
  setSelectedFormat,
}: {
  availableFormats: CcSwitchFormat[];
  selectedFormat: CcSwitchFormat;
  setSelectedFormat: (format: CcSwitchFormat) => void;
}) {
  if (availableFormats.length === 0 || availableFormats.includes(selectedFormat)) {
    return;
  }
  setSelectedFormat(availableFormats[0]);
}

function syncSelectedModel({
  selectedFormatModels,
  selectedModelId,
  setSelectedModelId,
}: {
  selectedFormatModels: { id: string }[];
  selectedModelId: string;
  setSelectedModelId: (modelId: string) => void;
}) {
  if (selectedFormatModels.some((model) => model.id === selectedModelId)) {
    return;
  }
  setSelectedModelId(selectedFormatModels[0]?.id ?? '');
}
