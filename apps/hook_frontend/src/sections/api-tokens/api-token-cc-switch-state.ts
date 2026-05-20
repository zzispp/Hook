'use client';

import type { ApiToken } from 'src/types/api-token';
import type { GlobalModelResponse } from 'src/types/model';
import type { TokenScope } from './api-token-management-types';
import type { CcSwitchFormat } from './api-token-cc-switch-utils';

import { useMemo, useState, useEffect } from 'react';

import {
  getApiTokenSecret,
  getAdminApiTokenSecret,
} from 'src/actions/api-tokens';

import {
  CC_SWITCH_FORMATS,
  tokenModelOptions,
  fetchTokenModelIds,
  buildCcSwitchDeepLink,
  modelsForCcSwitchFormat,
} from './api-token-cc-switch-utils';

type CcSwitchStep = 'format' | 'model';

type UseCcSwitchImportDialogArgs = {
  scope: TokenScope;
  t: (key: string, options?: Record<string, unknown>) => string;
  catalog: GlobalModelResponse[];
  siteName?: string | null;
};

export type CcSwitchImportDialogState = ReturnType<typeof useCcSwitchImportDialog>;

export function useCcSwitchImportDialog({
  scope,
  t,
  catalog,
  siteName,
}: UseCcSwitchImportDialogArgs) {
  const [open, setOpen] = useState(false);
  const [step, setStep] = useState<CcSwitchStep>('format');
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState(false);
  const [targetToken, setTargetToken] = useState<ApiToken | null>(null);
  const [rawToken, setRawToken] = useState('');
  const [modelIds, setModelIds] = useState<string[]>([]);
  const [selectedFormat, setSelectedFormat] = useState<CcSwitchFormat>('claude');
  const [selectedModelId, setSelectedModelId] = useState('');
  const [error, setError] = useState<string | null>(null);

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

  async function openImport(token: ApiToken) {
    resetDialog();
    setOpen(true);
    setTargetToken(token);
    await loadTokenContext({
      scope,
      tokenId: token.id,
      setLoading,
      setRawToken,
      setModelIds,
      setError,
      t,
    });
  }

  function closeDialog() {
    resetDialog();
  }

  function resetDialog() {
    setOpen(false);
    setStep('format');
    setLoading(false);
    setImporting(false);
    setTargetToken(null);
    setRawToken('');
    setModelIds([]);
    setSelectedFormat('claude');
    setSelectedModelId('');
    setError(null);
  }

  function backToFormat() {
    setStep('format');
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
    if (!targetToken) return;
    await loadTokenContext({
      scope,
      tokenId: targetToken.id,
      setLoading,
      setRawToken,
      setModelIds,
      setError,
      t,
    });
  }

  function importToCcSwitch() {
    if (!targetToken || !rawToken || !selectedModelId) {
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
    backToFormat,
    canContinueToModel: selectedFormatModels.length > 0 && !loading,
    canImport: !!rawToken && !!selectedModelId && !loading && !importing,
    closeDialog,
    error,
    importing,
    loading,
    modelOptions,
    open,
    openImport,
    goToModelStep,
    reload,
    selectedFormat,
    selectedFormatModels,
    selectedModelId,
    setSelectedFormat,
    setSelectedModelId,
    step,
    targetToken,
    importToCcSwitch,
  };
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

type LoadTokenContextArgs = {
  scope: TokenScope;
  tokenId: string;
  setLoading: (loading: boolean) => void;
  setRawToken: (rawToken: string) => void;
  setModelIds: (modelIds: string[]) => void;
  setError: (error: string | null) => void;
  t: (key: string, options?: Record<string, unknown>) => string;
};

async function loadTokenContext({
  scope,
  tokenId,
  setLoading,
  setRawToken,
  setModelIds,
  setError,
  t,
}: LoadTokenContextArgs) {
  setLoading(true);
  setError(null);
  try {
    const secret = await loadRawToken(scope, tokenId);
    const ids = await fetchTokenModelIds(secret);
    applyLoadedContext(secret, ids, setRawToken, setModelIds, setError, t);
  } catch (error) {
    setError(error instanceof Error ? error.message : t('tokens.ccSwitch.loadFailed'));
    setRawToken('');
    setModelIds([]);
  } finally {
    setLoading(false);
  }
}

function applyLoadedContext(
  secret: string,
  modelIds: string[],
  setRawToken: (rawToken: string) => void,
  setModelIds: (modelIds: string[]) => void,
  setError: (error: string | null) => void,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  setRawToken(secret);
  setModelIds(modelIds);
  if (modelIds.length === 0) {
    setError(t('tokens.ccSwitch.noAvailableModels'));
  }
}

async function loadRawToken(scope: TokenScope, tokenId: string) {
  const response =
    scope === 'admin'
      ? await getAdminApiTokenSecret(tokenId)
      : await getApiTokenSecret(tokenId);
  return response.raw_token.trim();
}
