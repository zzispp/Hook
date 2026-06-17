import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderModelCost, ProviderModelBinding } from 'src/types/provider';

import { globalDefaultMode, globalDefaultTokenDraft, globalDefaultRequestPrice } from './provider-model-cost-utils';

const PRICE_COMPARE_PRECISION = 2;

export type CostRatioDetail = {
  fieldKey:
    | 'price_per_request'
    | 'input_price_per_million'
    | 'output_price_per_million'
    | 'cache_creation_price_per_million'
    | 'cache_read_price_per_million';
  labelKey: string;
  globalPrice: number | null;
  configuredPrice: number | null;
  ratio: number | null;
  formattedRatio: string | null;
  unavailableReasonKey?: 'missing_global' | 'missing_configured' | 'non_positive_global' | 'non_positive_configured';
};

export type CostRatioInfo = {
  label: string;
  approximateLabel?: string;
  reasonKey: 'uniform' | 'conflict' | 'unavailable';
  details: CostRatioDetail[];
};

export function costRatioInfo({
  binding,
  cost,
  models,
  t,
}: {
  binding: ProviderModelBinding;
  cost?: ProviderModelCost;
  models: GlobalModelResponse[];
  t: (key: string, options?: Record<string, unknown>) => string;
}): CostRatioInfo {
  if (!cost) {
    return {
      label: '1.00x',
      reasonKey: 'uniform',
      details: globalDefaultRatioDetails(binding, models),
    };
  }

  const details = cost.cost_mode === 'per_request'
    ? perRequestRatioDetails(binding, cost, models)
    : perTokenRatioDetails(binding, cost, models);
  const comparable = details.filter((detail) => detail.formattedRatio);

  if (comparable.length === 0) {
    return {
      label: t('providers.customCostMultiplier'),
      reasonKey: 'unavailable',
      details,
    };
  }

  const uniqueRatios = new Set(comparable.map((detail) => detail.formattedRatio));
  if (uniqueRatios.size === 1) {
    return {
      label: `${comparable[0].formattedRatio}x`,
      reasonKey: 'uniform',
      details,
    };
  }

  const approximate = approximateRatioLabel(comparable);
  return {
    label: `${t('providers.customCostMultiplier')} ${t('providers.multiplierApprox', { ratio: approximate })}`,
    approximateLabel: t('providers.multiplierApprox', { ratio: approximate }),
    reasonKey: 'conflict',
    details,
  };
}

function globalDefaultRatioDetails(
  binding: ProviderModelBinding,
  models: GlobalModelResponse[]
) {
  const mode = globalDefaultMode(binding, models);
  if (mode === 'per_request') {
    const price = globalDefaultRequestPrice(binding, models);
    return [
      ratioDetail({
        fieldKey: 'price_per_request',
        labelKey: 'providers.pricePerRequest',
        globalPrice: price,
        configuredPrice: price,
      }),
    ];
  }

  const globalDraft = globalDefaultTokenDraft(binding, models);
  return [
    ratioDetail({
      fieldKey: 'input_price_per_million',
      labelKey: 'requestRecords.inputPrice',
      globalPrice: optionalNumber(globalDraft.input_price_per_million),
      configuredPrice: optionalNumber(globalDraft.input_price_per_million),
    }),
    ratioDetail({
      fieldKey: 'output_price_per_million',
      labelKey: 'requestRecords.outputPrice',
      globalPrice: optionalNumber(globalDraft.output_price_per_million),
      configuredPrice: optionalNumber(globalDraft.output_price_per_million),
    }),
    ratioDetail({
      fieldKey: 'cache_creation_price_per_million',
      labelKey: 'requestRecords.cacheCreationPrice',
      globalPrice: optionalNumber(globalDraft.cache_creation_price_per_million),
      configuredPrice: optionalNumber(globalDraft.cache_creation_price_per_million),
    }),
    ratioDetail({
      fieldKey: 'cache_read_price_per_million',
      labelKey: 'requestRecords.cacheReadPrice',
      globalPrice: optionalNumber(globalDraft.cache_read_price_per_million),
      configuredPrice: optionalNumber(globalDraft.cache_read_price_per_million),
    }),
  ];
}

function perRequestRatioDetails(
  binding: ProviderModelBinding,
  cost: ProviderModelCost,
  models: GlobalModelResponse[]
) {
  return [
    ratioDetail({
      fieldKey: 'price_per_request',
      labelKey: 'providers.pricePerRequest',
      globalPrice: globalDefaultRequestPrice(binding, models),
      configuredPrice: cost.price_per_request ?? null,
    }),
  ];
}

function perTokenRatioDetails(
  binding: ProviderModelBinding,
  cost: ProviderModelCost,
  models: GlobalModelResponse[]
) {
  const globalDraft = globalDefaultTokenDraft(binding, models);
  return [
    ratioDetail({
      fieldKey: 'input_price_per_million',
      labelKey: 'requestRecords.inputPrice',
      globalPrice: optionalNumber(globalDraft.input_price_per_million),
      configuredPrice: cost.input_price_per_million ?? null,
    }),
    ratioDetail({
      fieldKey: 'output_price_per_million',
      labelKey: 'requestRecords.outputPrice',
      globalPrice: optionalNumber(globalDraft.output_price_per_million),
      configuredPrice: cost.output_price_per_million ?? null,
    }),
    ratioDetail({
      fieldKey: 'cache_creation_price_per_million',
      labelKey: 'requestRecords.cacheCreationPrice',
      globalPrice: optionalNumber(globalDraft.cache_creation_price_per_million),
      configuredPrice: cost.cache_creation_price_per_million ?? null,
    }),
    ratioDetail({
      fieldKey: 'cache_read_price_per_million',
      labelKey: 'requestRecords.cacheReadPrice',
      globalPrice: optionalNumber(globalDraft.cache_read_price_per_million),
      configuredPrice: cost.cache_read_price_per_million ?? null,
    }),
  ];
}

function ratioDetail({
  fieldKey,
  labelKey,
  globalPrice,
  configuredPrice,
}: {
  fieldKey: CostRatioDetail['fieldKey'];
  labelKey: string;
  globalPrice: number | null;
  configuredPrice: number | null;
}): CostRatioDetail {
  if (globalPrice === null || globalPrice === undefined) {
    return { fieldKey, labelKey, globalPrice: null, configuredPrice, ratio: null, formattedRatio: null, unavailableReasonKey: 'missing_global' };
  }
  if (configuredPrice === null || configuredPrice === undefined) {
    return { fieldKey, labelKey, globalPrice, configuredPrice: null, ratio: null, formattedRatio: null, unavailableReasonKey: 'missing_configured' };
  }
  if (globalPrice <= 0) {
    return { fieldKey, labelKey, globalPrice, configuredPrice, ratio: null, formattedRatio: null, unavailableReasonKey: 'non_positive_global' };
  }
  if (configuredPrice <= 0) {
    return { fieldKey, labelKey, globalPrice, configuredPrice, ratio: null, formattedRatio: null, unavailableReasonKey: 'non_positive_configured' };
  }

  const ratio = configuredPrice / globalPrice;
  return {
    fieldKey,
    labelKey,
    globalPrice,
    configuredPrice,
    ratio,
    formattedRatio: formatRatioValue(ratio),
  };
}

function approximateRatioLabel(details: CostRatioDetail[]) {
  const values = details
    .map((detail) => detail.ratio)
    .filter((value): value is number => value !== null && Number.isFinite(value))
    .sort((left, right) => left - right);
  const middle = Math.floor(values.length / 2);
  const median = values.length % 2 === 0 ? (values[middle - 1] + values[middle]) / 2 : values[middle];
  return formatRatioValue(median);
}

function formatRatioValue(value: number) {
  return value.toFixed(PRICE_COMPARE_PRECISION);
}

function optionalNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : null;
}
