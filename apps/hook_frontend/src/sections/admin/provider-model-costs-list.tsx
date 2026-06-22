'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ModelCostRowItem, ModelCostSectionItem } from './provider-model-cost-types';
import type { ProviderApiKey, ProviderModelCost, ProviderModelBinding } from 'src/types/provider';

import Box from '@mui/material/Box';
import Accordion from '@mui/material/Accordion';
import Typography from '@mui/material/Typography';
import AccordionDetails from '@mui/material/AccordionDetails';
import AccordionSummary, { accordionSummaryClasses } from '@mui/material/AccordionSummary';

import { useTranslate } from 'src/locales/use-locales';

import { ProviderModelCostsRow } from './provider-model-costs-row';
import {
  bindingLabel,
  globalDefaultMode,
  keyModelScopeLabel,
  effectiveTokenDraft,
  bindingsAllowedForKey,
  effectiveRequestPrice,
} from './provider-model-cost-utils';

type ToggleExpanded = (keyId: string, expanded: boolean) => void;

export function ProviderModelCostsList({
  expandedKeyIds,
  hideUnconfigured,
  providerId,
  sections,
  onExpandedChange,
}: {
  expandedKeyIds: string[];
  hideUnconfigured: boolean;
  providerId: string;
  sections: ModelCostSectionItem[];
  onExpandedChange: ToggleExpanded;
}) {
  return sections.map((section) => (
    <CostSection
      key={section.key.id}
      expanded={expandedKeyIds.includes(section.key.id)}
      hideUnconfigured={hideUnconfigured}
      providerId={providerId}
      section={section}
      onExpandedChange={onExpandedChange}
    />
  ));
}

export function buildModelCostSections({
  apiKeys,
  bindings,
  costs,
  models,
}: {
  apiKeys: ProviderApiKey[];
  bindings: ProviderModelBinding[];
  costs: ProviderModelCost[];
  models: GlobalModelResponse[];
}) {
  const costMap = new Map(costs.map((cost) => [`${cost.key_id}:${cost.provider_model_id}`, cost]));
  return apiKeys.map((key) => {
    const allowedBindings = bindingsAllowedForKey(key, bindings);
    const rows = allowedBindings.map((binding) => buildCostRow({ key, binding, costMap, models }));
    return {
      key,
      rows,
      totalModelCount: rows.length,
      configuredModelCount: rows.filter((row) => Boolean(row.cost)).length,
    };
  });
}

function buildCostRow({
  binding,
  costMap,
  key,
  models,
}: {
  binding: ProviderModelBinding;
  costMap: Map<string, ProviderModelCost>;
  key: ProviderApiKey;
  models: GlobalModelResponse[];
}): ModelCostRowItem {
  const cost = costMap.get(`${key.id}:${binding.id}`);
  return {
    key,
    binding,
    cost,
    models,
    modelLabel: bindingLabel(binding, models),
    mode: cost?.cost_mode ?? globalDefaultMode(binding, models),
    source: cost ? 'configured' : 'global_default',
    requestPrice: effectiveRequestPrice(binding, models, cost),
    tokenDraft: effectiveTokenDraft(binding, models, cost),
  };
}

function CostSection({
  expanded,
  hideUnconfigured,
  providerId,
  section,
  onExpandedChange,
}: {
  expanded: boolean;
  hideUnconfigured: boolean;
  providerId: string;
  section: ModelCostSectionItem;
  onExpandedChange: ToggleExpanded;
}) {
  const { t } = useTranslate('admin');
  const rows = hideUnconfigured ? section.rows.filter((row) => Boolean(row.cost)) : section.rows;
  const summary = `${section.key.name} (${section.configuredModelCount}/${section.totalModelCount})`;

  return (
    <Accordion
      disableGutters
      expanded={expanded}
      sx={sectionAccordionSx}
      onChange={(_event, nextExpanded) => onExpandedChange(section.key.id, nextExpanded)}
    >
      <AccordionSummary>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap>
            {summary}
          </Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {keyModelScopeLabel(section.key, t)}
          </Typography>
        </Box>
      </AccordionSummary>
      <AccordionDetails sx={sectionDetailsSx}>
        {rows.length > 0 ? (
          <Box sx={sectionListSx}>
            {rows.map((row) => (
              <ProviderModelCostsRow key={row.binding.id} providerId={providerId} row={row} />
            ))}
          </Box>
        ) : (
          <Typography variant="body2" color="text.secondary" sx={{ px: 2, pb: 2 }}>
            {t('providers.noConfiguredModelCosts')}
          </Typography>
        )}
      </AccordionDetails>
    </Accordion>
  );
}

const sectionAccordionSx = { bgcolor: 'transparent', [`& .${accordionSummaryClasses.root}`]: { px: 2 } };
const sectionDetailsSx = { p: 0 };
const sectionListSx = { '& > * + *': { borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` } };
