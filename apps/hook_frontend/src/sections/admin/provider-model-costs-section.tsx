'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderModelCost, ProviderModelBinding } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';
import { ProviderModelCostDialog } from './provider-model-cost-dialog';
import { ProviderModelCostsList, buildModelCostSections } from './provider-model-costs-list';

type Props = {
  providerId: string;
  apiKeys: ProviderApiKey[];
  bindings: ProviderModelBinding[];
  costs: ProviderModelCost[];
  loading: boolean;
  models: GlobalModelResponse[];
};

export function ProviderModelCostsSection({ providerId, apiKeys, bindings, costs, loading, models }: Props) {
  const { t } = useTranslate('admin');
  const state = useModelCostSectionState(providerId);
  const sections = useMemo(() => buildModelCostSections({ apiKeys, bindings, costs, models }), [apiKeys, bindings, costs, models]);
  const rowCount = useMemo(() => sections.reduce((total, section) => total + section.totalModelCount, 0), [sections]);

  return (
    <>
      <Box sx={panelSx}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
          <Typography variant="subtitle2">{t('providers.modelCosts')}</Typography>
          <Stack direction="row" spacing={1} alignItems="center" useFlexGap flexWrap="wrap" justifyContent="flex-end">
            <FormControlLabel
              sx={switchLabelSx}
              control={
                <Switch
                  size="small"
                  checked={state.hideUnconfigured}
                  onChange={(event) => state.setHideUnconfigured(event.target.checked)}
                />
              }
              label={t('providers.hideUnconfiguredModelCosts')}
            />
            <Button
              color="inherit"
              size="small"
              variant="outlined"
              disabled={apiKeys.length === 0 || bindings.length === 0}
              startIcon={<Iconify icon="mingcute:add-line" width={14} />}
              onClick={() => state.setDialogOpen(true)}
            >
              {t('actions.addProviderModelCost')}
            </Button>
          </Stack>
        </Stack>
        <Box sx={listSx}>
          <ProviderModelCostsList
            expandedKeyIds={state.expandedKeyIds}
            hideUnconfigured={state.hideUnconfigured}
            providerId={providerId}
            sections={sections}
            onExpandedChange={state.toggleExpanded}
          />
          <EmptyList loading={loading} length={rowCount} />
        </Box>
      </Box>
      <ProviderModelCostDialog
        open={state.dialogOpen}
        providerId={providerId}
        apiKeys={apiKeys}
        bindings={bindings}
        models={models}
        onClose={() => state.setDialogOpen(false)}
      />
    </>
  );
}

function useModelCostSectionState(providerId: string) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [hideUnconfigured, setHideUnconfigured] = useState(true);
  const [expandedKeyIds, setExpandedKeyIds] = useState<string[]>([]);

  useEffect(() => {
    setDialogOpen(false);
    setHideUnconfigured(true);
    setExpandedKeyIds([]);
  }, [providerId]);
  return {
    dialogOpen,
    hideUnconfigured,
    expandedKeyIds,
    setDialogOpen,
    setHideUnconfigured,
    toggleExpanded: (keyId: string, expanded: boolean) => {
      setExpandedKeyIds((current) => (expanded ? [...current, keyId] : current.filter((id) => id !== keyId)));
    },
  };
}

const panelSx = { border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 2, overflow: 'hidden' };
const headerSx = { p: 2, borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` };
const listSx = { '& > * + *': { borderTop: (theme: Theme) => `1px solid ${theme.vars.palette.divider}` } };
const switchLabelSx = { mr: 0, '& .MuiFormControlLabel-label': { typography: 'body2' } };
