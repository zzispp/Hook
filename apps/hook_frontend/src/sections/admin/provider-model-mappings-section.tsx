'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderModelBinding } from 'src/types/provider';

import { useMemo, useState } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';
import { updateProviderModel } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { EmptyList } from './provider-bindings-shared';
import { bindingDisplayLabel } from './provider-model-mapping-utils';
import { ProviderModelMappingDialog } from './provider-model-mapping-dialog';

type Props = {
  providerId: string;
  items: ProviderModelBinding[];
  loading: boolean;
  models: GlobalModelResponse[];
};

export function ProviderModelMappingsSection({ providerId, items, loading, models }: Props) {
  const { t } = useTranslate('admin');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingBinding, setEditingBinding] = useState<ProviderModelBinding | null>(null);
  const mappedItems = useMemo(
    () =>
      items
        .filter((item) => Boolean(item.provider_model_mapping))
        .sort((left, right) => bindingDisplayLabel(left, models).localeCompare(bindingDisplayLabel(right, models))),
    [items, models]
  );

  return (
    <>
      <Box sx={panelSx}>
        <Stack direction="row" alignItems="center" justifyContent="space-between" sx={headerSx}>
          <Typography variant="subtitle2">{t('providers.modelMappings')}</Typography>
          <Button
            color="inherit"
            variant="outlined"
            size="small"
            startIcon={<Iconify icon="solar:tag-horizontal-bold-duotone" />}
            disabled={items.length === 0}
            onClick={() => setDialogOpen(true)}
          >
            {t('actions.addProviderModelMapping')}
          </Button>
        </Stack>
        <Divider />
        <Box sx={{ p: 2 }}>
          {mappedItems.length > 0 ? (
            <Stack spacing={1.5}>
              {mappedItems.map((item) => (
                <MappingRow
                  key={item.id}
                  binding={item}
                  label={bindingDisplayLabel(item, models)}
                  providerId={providerId}
                  onEdit={() => {
                    setEditingBinding(item);
                    setDialogOpen(true);
                  }}
                />
              ))}
            </Stack>
          ) : (
            <EmptyList loading={loading} length={mappedItems.length} />
          )}
        </Box>
      </Box>
      <ProviderModelMappingDialog
        open={dialogOpen}
        providerId={providerId}
        items={items}
        models={models}
        editingBinding={editingBinding}
        onClose={() => {
          setDialogOpen(false);
          setEditingBinding(null);
        }}
        onSaved={() => setEditingBinding(null)}
      />
    </>
  );
}

function MappingRow({
  binding,
  label,
  providerId,
  onEdit,
}: {
  binding: ProviderModelBinding;
  label: string;
  providerId: string;
  onEdit: () => void;
}) {
  const { t } = useTranslate('admin');

  const clearMappings = async () => {
    try {
      await updateProviderModel(providerId, binding.id, { provider_model_mapping: null });
      toast.success(t('messages.providerModelMappingDeleted'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  };

  return (
    <Box sx={rowSx}>
      <Stack direction="row" justifyContent="space-between" spacing={1} alignItems="flex-start">
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2" noWrap>
            {label}
          </Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary', fontFamily: 'monospace' }}>
            {binding.provider_model_name}
          </Typography>
        </Box>
        <Stack direction="row" spacing={0.5}>
          <IconButton size="small" onClick={onEdit}>
            <Iconify icon="solar:pen-bold" width={16} />
          </IconButton>
          <IconButton size="small" onClick={() => void clearMappings()}>
            <Iconify icon="solar:trash-bin-trash-bold" width={16} />
          </IconButton>
        </Stack>
      </Stack>
      <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap sx={{ mt: 1.5 }}>
        {binding.provider_model_mapping ? (
          <>
            <Chip size="small" label={binding.provider_model_mapping.name} sx={{ fontFamily: 'monospace' }} />
            {binding.provider_model_mapping.reasoning_effort ? (
              <Chip size="small" label={`${t('providers.reasoningEffort')}: ${binding.provider_model_mapping.reasoning_effort}`} />
            ) : null}
          </>
        ) : null}
      </Stack>
    </Box>
  );
}

const panelSx = { border: (theme: any) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1 };
const headerSx = { px: 2, py: 1.5 };
const rowSx = { border: (theme: any) => `1px solid ${theme.vars.palette.divider}`, borderRadius: 1.5, p: 1.5 };
