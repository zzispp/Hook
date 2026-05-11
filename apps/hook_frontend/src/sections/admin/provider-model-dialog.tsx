'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { useProviderChildDialogs } from './provider-management-state';

import MenuItem from '@mui/material/MenuItem';

import { useTranslate } from 'src/locales/use-locales';

import { TextFieldRow, ManagementDialog } from './shared';

export function ProviderModelDialog({
  dialogs,
  models,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
}) {
  const { t } = useTranslate('admin');
  const hasModels = models.length > 0;

  return (
    <ManagementDialog
      open={dialogs.modelOpen}
      title={t('dialogs.createProviderModel')}
      submitting={dialogs.submitting}
      submitDisabled={!hasModels || !dialogs.modelForm.global_model_id}
      onClose={dialogs.closeModel}
      onSubmit={dialogs.submitModel}
    >
      <ProviderModelMainFields dialogs={dialogs} models={models} />
    </ManagementDialog>
  );
}

function ProviderModelMainFields({
  dialogs,
  models,
}: {
  dialogs: ReturnType<typeof useProviderChildDialogs>;
  models: Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>[];
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      select
      required
      label={t('fields.model')}
      value={dialogs.modelForm.global_model_id}
      SelectProps={{ displayEmpty: true }}
      onChange={(value) =>
        dialogs.setModelForm((form) => ({
          ...form,
          global_model_id: value,
          provider_model_name: models.find((model) => model.id === value)?.name ?? '',
        }))
      }
    >
      {models.length ? (
        models.map((model) => (
          <MenuItem key={model.id} value={model.id}>
            {model.display_name || model.name}
          </MenuItem>
        ))
      ) : (
        <MenuItem disabled value="">
          {t('providers.noBindableModels')}
        </MenuItem>
      )}
    </TextFieldRow>
  );
}
