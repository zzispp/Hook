'use client';

import type { Theme } from '@mui/material/styles';
import type { GlobalModelResponse } from 'src/types/model';
import type { ProviderModelBinding, ProviderModelReasoningEffort } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogActions from '@mui/material/DialogActions';

import { useTranslate } from 'src/locales/use-locales';
import { updateProviderModel, fetchProviderUpstreamModels } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import {
  ClientModelField,
  ProviderModelsField,
  ReasoningEffortField,
} from './provider-model-mapping-fields';
import {
  toggleName,
  mappingName,
  matchesQuery,
  allowCustomName,
  mappingReasoningEffort,
} from './provider-model-mapping-utils';

type Props = {
  open: boolean;
  providerId: string;
  items: ProviderModelBinding[];
  models: GlobalModelResponse[];
  editingBinding: ProviderModelBinding | null;
  onClose: () => void;
  onSaved: () => void;
};

export function ProviderModelMappingDialog(props: Props) {
  const { t } = useTranslate('admin');
  const [selectedBindingId, setSelectedBindingId] = useState('');
  const [selectedName, setSelectedName] = useState('');
  const [reasoningEffort, setReasoningEffort] = useState<ProviderModelReasoningEffort | ''>('');
  const [searchQuery, setSearchQuery] = useState('');
  const [upstreamModels, setUpstreamModels] = useState<string[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [saving, setSaving] = useState(false);

  const currentBinding = useMemo(
    () => props.items.find((item) => item.id === selectedBindingId) ?? null,
    [props.items, selectedBindingId]
  );
  const upstreamSet = useMemo(() => new Set(upstreamModels), [upstreamModels]);
  const customNames = useMemo(() => {
    const name = selectedName.trim();
    return name && !upstreamSet.has(name) && matchesQuery(searchQuery)(name) ? [name] : [];
  }, [searchQuery, selectedName, upstreamSet]);
  const filteredUpstreamModels = useMemo(
    () => upstreamModels.filter(matchesQuery(searchQuery)),
    [searchQuery, upstreamModels]
  );
  const canAddCustom = useMemo(
    () => allowCustomName(searchQuery, selectedName, upstreamSet),
    [searchQuery, selectedName, upstreamSet]
  );

  useEffect(() => {
    if (!props.open) return;
    const binding = props.editingBinding;
    setSelectedBindingId(binding?.id ?? '');
    setSelectedName(mappingName(binding));
    setReasoningEffort(mappingReasoningEffort(binding));
    setSearchQuery('');
    setUpstreamModels([]);
  }, [props.editingBinding, props.open]);

  const submit = async () => {
    if (!currentBinding || saving || !selectedName.trim()) return;
    setSaving(true);
    try {
      await updateProviderModel(props.providerId, currentBinding.id, {
        provider_model_mapping: {
          name: selectedName.trim(),
          ...(reasoningEffort ? { reasoning_effort: reasoningEffort } : {}),
        },
      });
      toast.success(
        props.editingBinding ? t('messages.providerModelMappingUpdated') : t('messages.providerModelMappingSaved')
      );
      props.onSaved();
      props.onClose();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="sm" open={props.open} onClose={props.onClose} PaperProps={{ sx: paperSx }}>
      <DialogHeader
        title={props.editingBinding ? t('dialogs.editProviderModelMapping') : t('dialogs.createProviderModelMapping')}
        description={t('providers.modelMappingDialogDescription')}
        onClose={props.onClose}
      />
      <Box sx={contentSx}>
        <ReasoningEffortField value={reasoningEffort} onChange={setReasoningEffort} />
        <ClientModelField
          disabled={!!props.editingBinding}
          items={props.items}
          models={props.models}
          value={selectedBindingId}
          onChange={(value) => {
            const binding = props.items.find((item) => item.id === value) ?? null;
            setSelectedBindingId(value);
            setSelectedName(mappingName(binding));
            setReasoningEffort(mappingReasoningEffort(binding));
          }}
        />
        <ProviderModelsField
          canAddCustom={canAddCustom}
          customNames={customNames}
          loading={loadingModels}
          query={searchQuery}
          reasoningEffort={reasoningEffort}
          selectedName={selectedName}
          upstreamModels={filteredUpstreamModels}
          onAddCustom={() => {
            const name = searchQuery.trim();
            if (!name) return;
            setSelectedName(name);
            setSearchQuery('');
          }}
          onFetch={async () => {
            setLoadingModels(true);
            try {
              setUpstreamModels(await fetchProviderUpstreamModels(props.providerId));
            } catch (error) {
              toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
            } finally {
              setLoadingModels(false);
            }
          }}
          onQueryChange={setSearchQuery}
          onToggleName={(name) => setSelectedName((current) => toggleName(current, name))}
        />
      </Box>
      <DialogActions sx={footerSx}>
        <Button variant="outlined" onClick={props.onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={saving} disabled={!currentBinding || !selectedName.trim()} onClick={submit}>
          {props.editingBinding ? t('common.save') : t('common.add')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DialogHeader({ title, description, onClose }: { title: string; description: string; onClose: () => void }) {
  return (
    <Stack direction="row" alignItems="center" spacing={1.5} sx={headerSx}>
      <Box sx={headerIconSx}>
        <Iconify icon="solar:tag-horizontal-bold-duotone" width={20} />
      </Box>
      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography variant="h6">{title}</Typography>
        <Typography variant="caption" color="text.secondary">
          {description}
        </Typography>
      </Box>
      <IconButton onClick={onClose}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

const paperSx = (theme: Theme) => ({ borderRadius: 2, border: `1px solid ${theme.vars.palette.divider}` });
const headerSx = { px: 2.5, py: 2, alignItems: 'center' };
const headerIconSx = {
  width: 40,
  height: 40,
  borderRadius: 1.5,
  display: 'grid',
  placeItems: 'center',
  bgcolor: 'primary.lighter',
  color: 'primary.main',
};
const contentSx = { px: 2.5, pb: 1.5, display: 'grid', gap: 2 };
const footerSx = { px: 2.5, py: 2 };
