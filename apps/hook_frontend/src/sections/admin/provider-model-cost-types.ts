import type { GlobalModelResponse } from 'src/types/model';
import type { TokenCostDraft } from './provider-model-cost-utils';
import type {
  ProviderApiKey,
  ProviderModelCost,
  ProviderModelBinding,
  ProviderModelCostMode,
} from 'src/types/provider';

export type ModelCostRowItem = {
  key: ProviderApiKey;
  binding: ProviderModelBinding;
  cost?: ProviderModelCost;
  models: GlobalModelResponse[];
  modelLabel: string;
  mode: ProviderModelCostMode;
  source: 'configured' | 'global_default';
  requestPrice: number | null | undefined;
  tokenDraft: TokenCostDraft;
};

export type ModelCostSectionItem = {
  key: ProviderApiKey;
  totalModelCount: number;
  configuredModelCount: number;
  rows: ModelCostRowItem[];
};
