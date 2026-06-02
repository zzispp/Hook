'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderApiKey, ProviderEndpoint, ProviderModelBinding } from 'src/types/provider';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import TableBody from '@mui/material/TableBody';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';
import { ProviderModelRow } from './provider-model-binding-row';
import { ProviderModelTestDialog } from './provider-model-test-dialog';
import { GlobalModelPriceDialog } from './provider-model-price-dialog';

type Props = {
  providerId: string;
  endpoints: ProviderEndpoint[];
  apiKeys: ProviderApiKey[];
  items: ProviderModelBinding[];
  loading: boolean;
  models: GlobalModelResponse[];
  onAssociate: () => void;
};

export function ProviderModelBindingsSection({
  providerId,
  endpoints,
  apiKeys,
  items,
  loading,
  models,
  onAssociate,
}: Props) {
  const { t } = useTranslate('admin');
  const [editingModel, setEditingModel] = useState<GlobalModelResponse | null>(null);
  const [testingBinding, setTestingBinding] = useState<ProviderModelBinding | null>(null);
  const sortedItems = [...items].sort(compareBindings(models));

  return (
    <Box sx={panelSx}>
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
        <Typography variant="subtitle2">{t('providers.modelList')}</Typography>
        <Button
          color="inherit"
          variant="outlined"
          size="small"
          startIcon={<Iconify icon="solar:list-bold" />}
          onClick={onAssociate}
        >
          {t('actions.associateProviderModels')}
        </Button>
      </Stack>
      <Box sx={{ overflow: 'hidden' }}>
        {sortedItems.length > 0 ? (
          <Table size="small" sx={{ tableLayout: 'fixed' }}>
            <colgroup>
              <col width="45%" />
              <col width="30%" />
              <col width="25%" />
            </colgroup>
            <TableBody>
              {sortedItems.map((binding) => (
                <ProviderModelRow
                  key={binding.id}
                  binding={binding}
                  providerId={providerId}
                  model={findGlobalModel(models, binding.global_model_id)}
                  onEdit={setEditingModel}
                  onTest={setTestingBinding}
                />
              ))}
            </TableBody>
          </Table>
        ) : (
          <EmptyList loading={loading} length={sortedItems.length} />
        )}
      </Box>
      <GlobalModelPriceDialog model={editingModel} onClose={() => setEditingModel(null)} />
      <ProviderModelTestDialog
        providerId={providerId}
        binding={testingBinding}
        endpoints={endpoints}
        apiKeys={apiKeys}
        onClose={() => setTestingBinding(null)}
      />
    </Box>
  );
}

function compareBindings(models: GlobalModelResponse[]) {
  return (left: ProviderModelBinding, right: ProviderModelBinding) =>
    modelName(left, models).localeCompare(modelName(right, models));
}

function modelName(binding: ProviderModelBinding, models: GlobalModelResponse[]) {
  const model = findGlobalModel(models, binding.global_model_id);
  return model?.display_name || binding.provider_model_name;
}

function findGlobalModel(models: GlobalModelResponse[], id: string) {
  return models.find((model) => model.id === id);
}

const panelSx = {
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
  borderRadius: 2,
  overflow: 'hidden',
};
const headerSx = {
  px: 2,
  py: 1.5,
  borderBottom: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};
