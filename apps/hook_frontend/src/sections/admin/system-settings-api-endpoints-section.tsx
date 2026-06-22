'use client';

import type { ApiEndpoint } from 'src/types/system-setting';
import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';
import {
  addApiEndpoint,
  moveApiEndpoint,
  removeApiEndpoint,
  updateApiEndpoint,
} from './system-settings-api-endpoints-utils';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function SystemSettingsApiEndpointsSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection
      title={t('systemSettings.sections.apiEndpoints')}
      description={t('systemSettings.apiEndpoints.description')}
    >
      <Stack spacing={2}>
        {form.api_endpoints.map((endpoint, index) => (
          <ApiEndpointEditor
            key={endpoint.id}
            endpoint={endpoint}
            index={index}
            total={form.api_endpoints.length}
            onChange={(next) => updateApiEndpoint(setForm, index, next)}
            onMove={(direction) => moveApiEndpoint(setForm, index, direction)}
            onRemove={() => removeApiEndpoint(setForm, index)}
          />
        ))}
        <Button
          variant="outlined"
          startIcon={<Iconify icon="mingcute:add-line" />}
          onClick={() => addApiEndpoint(setForm)}
        >
          {t('systemSettings.apiEndpoints.add')}
        </Button>
      </Stack>
    </SettingsSection>
  );
}

function ApiEndpointEditor({
  endpoint,
  index,
  total,
  onChange,
  onMove,
  onRemove,
}: {
  endpoint: ApiEndpoint;
  index: number;
  total: number;
  onChange: (endpoint: ApiEndpoint) => void;
  onMove: (direction: -1 | 1) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2} sx={{ p: 2, border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
      <ApiEndpointHeader index={index} total={total} onMove={onMove} onRemove={onRemove} />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('systemSettings.fields.apiEndpointName')}
          value={endpoint.name}
          onChange={(value) => onChange({ ...endpoint, name: value })}
        />
        <TextFieldRow
          required
          label={t('systemSettings.fields.apiEndpointUrl')}
          value={endpoint.url}
          placeholder="https://api.example.com"
          onChange={(value) => onChange({ ...endpoint, url: value })}
        />
      </Stack>
      <TextFieldRow
        label={t('systemSettings.fields.apiEndpointDescription')}
        value={endpoint.description}
        rows={2}
        onChange={(value) => onChange({ ...endpoint, description: value })}
      />
    </Stack>
  );
}

function ApiEndpointHeader({
  index,
  total,
  onMove,
  onRemove,
}: {
  index: number;
  total: number;
  onMove: (direction: -1 | 1) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack direction="row" alignItems="center" spacing={1}>
      <Typography variant="subtitle2" sx={{ flexGrow: 1 }}>
        {t('systemSettings.apiEndpoints.itemTitle', { index: index + 1 })}
      </Typography>
      <IconButton size="small" disabled={index === 0} onClick={() => onMove(-1)}>
        <Iconify icon="eva:arrow-upward-fill" width={18} />
      </IconButton>
      <IconButton size="small" disabled={index === total - 1} onClick={() => onMove(1)}>
        <Iconify icon="eva:arrow-downward-fill" width={18} />
      </IconButton>
      <IconButton size="small" color="error" onClick={onRemove}>
        <Iconify icon="solar:trash-bin-trash-bold" width={18} />
      </IconButton>
    </Stack>
  );
}
