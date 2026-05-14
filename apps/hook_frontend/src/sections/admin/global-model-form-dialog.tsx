'use client';

import type { GlobalModelForm } from './model-management-utils';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { accountingCurrencyLabel } from 'src/utils/money-boundary';

import { useTranslate } from 'src/locales/use-locales';

import { SwitchRow, TextFieldRow } from './shared';
import { CAPABILITY_KEYS } from './model-management-utils';
import { TieredPricingEditor } from './tiered-pricing-editor';

// ----------------------------------------------------------------------

type Props = {
  open: boolean;
  title: string;
  form: GlobalModelForm;
  isEdit: boolean;
  submitting: boolean;
  picker?: React.ReactNode;
  onClose: () => void;
  onSubmit: () => void;
  onChange: (form: GlobalModelForm) => void;
};

export function GlobalModelFormDialog({
  open,
  title,
  form,
  isEdit,
  submitting,
  picker,
  onClose,
  onSubmit,
  onChange,
}: Props) {
  const { t } = useTranslate('admin');

  return (
    <Dialog fullWidth maxWidth="lg" open={open} onClose={onClose}>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <Stack
          sx={{
            pt: 1,
            gap: 3,
            display: 'grid',
            gridTemplateColumns: { xs: '1fr', md: picker ? '320px 1fr' : '1fr' },
          }}
        >
          {picker}
          <Stack sx={{ gap: 2.5, minWidth: 0 }}>
            <BasicFields form={form} isEdit={isEdit} onChange={onChange} />
            <PricingFields form={form} onChange={onChange} />
            <MetadataFields form={form} onChange={onChange} />
            <CapabilityFields form={form} onChange={onChange} />
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function BasicFields({
  form,
  isEdit,
  onChange,
}: {
  form: GlobalModelForm;
  isEdit: boolean;
  onChange: (form: GlobalModelForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Section title={t('models.basic')}>
      <TextFieldRow
        required
        disabled={isEdit}
        label={t('fields.modelId')}
        value={form.name}
        onChange={(name) => onChange({ ...form, name })}
      />
      <TextFieldRow
        required
        label={t('fields.displayName')}
        value={form.display_name}
        onChange={(displayName) => onChange({ ...form, display_name: displayName })}
      />
      <TextFieldRow
        label={t('common.description')}
        value={form.description}
        onChange={(description) => onChange({ ...form, description })}
      />
      <SwitchRow
        label={t('common.active')}
        checked={form.is_active}
        onChange={(isActive) => onChange({ ...form, is_active: isActive })}
      />
    </Section>
  );
}

function PricingFields({
  form,
  onChange,
}: {
  form: GlobalModelForm;
  onChange: (form: GlobalModelForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack sx={{ gap: 2 }}>
      <Typography variant="subtitle2">{t('models.pricing')}</Typography>
      <Divider />
      <TieredPricingEditor
        pricing={form.default_tiered_pricing}
        onChange={(defaultTieredPricing) =>
          onChange({ ...form, default_tiered_pricing: defaultTieredPricing })
        }
      />
      <TextFieldRow
        type="number"
        label={accountingCurrencyLabel(t('fields.pricePerRequest'))}
        value={form.default_price_per_request}
        onChange={(value) => onChange({ ...form, default_price_per_request: value })}
      />
    </Stack>
  );
}

function MetadataFields({
  form,
  onChange,
}: {
  form: GlobalModelForm;
  onChange: (form: GlobalModelForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Section title={t('models.metadata')}>
      <TextFieldRow
        type="number"
        label={t('fields.contextLimit')}
        value={form.context_limit}
        onChange={(contextLimit) => onChange({ ...form, context_limit: contextLimit })}
      />
      <TextFieldRow
        type="number"
        label={t('fields.outputLimit')}
        value={form.output_limit}
        onChange={(outputLimit) => onChange({ ...form, output_limit: outputLimit })}
      />
      <TextFieldRow
        label={t('fields.family')}
        value={form.family}
        onChange={(family) => onChange({ ...form, family })}
      />
      <TextFieldRow
        label={t('fields.knowledgeCutoff')}
        value={form.knowledge_cutoff}
        onChange={(knowledgeCutoff) => onChange({ ...form, knowledge_cutoff: knowledgeCutoff })}
      />
      <TextFieldRow
        label={t('fields.releaseDate')}
        value={form.release_date}
        onChange={(releaseDate) => onChange({ ...form, release_date: releaseDate })}
      />
      <TextField
        fullWidth
        multiline
        minRows={2}
        label={t('fields.inputModalities')}
        value={form.input_modalities.join(', ')}
        onChange={(event) =>
          onChange({ ...form, input_modalities: parseList(event.target.value) })
        }
      />
      <TextField
        fullWidth
        multiline
        minRows={2}
        label={t('fields.outputModalities')}
        value={form.output_modalities.join(', ')}
        onChange={(event) =>
          onChange({ ...form, output_modalities: parseList(event.target.value) })
        }
      />
    </Section>
  );
}

function CapabilityFields({
  form,
  onChange,
}: {
  form: GlobalModelForm;
  onChange: (form: GlobalModelForm) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Section title={t('models.capabilities')}>
      {CAPABILITY_KEYS.map((key) => (
        <SwitchRow
          key={key}
          label={t(`models.capability.${key}`)}
          checked={capabilityValue(form, key)}
          onChange={(checked) => onChange(updateCapability(form, key, checked))}
        />
      ))}
    </Section>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <Stack sx={{ gap: 2 }}>
      <Typography variant="subtitle2">{title}</Typography>
      <Divider />
      <Stack sx={{ display: 'grid', gridTemplateColumns: { xs: '1fr', sm: '1fr 1fr' }, gap: 2 }}>
        {children}
      </Stack>
    </Stack>
  );
}

function parseList(value: string) {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

function capabilityValue(form: GlobalModelForm, key: (typeof CAPABILITY_KEYS)[number]) {
  if (key === 'vision') return form.supports_vision;
  if (key === 'function_calling') return form.supports_function_calling;
  if (key === 'streaming') return form.supports_streaming;
  if (key === 'extended_thinking') return form.supports_extended_thinking;
  if (key === 'structured_output') return form.supports_structured_output;
  if (key === 'temperature') return form.supports_temperature;
  if (key === 'attachment') return form.supports_attachment;
  return form.open_weights;
}

function updateCapability(
  form: GlobalModelForm,
  key: (typeof CAPABILITY_KEYS)[number],
  checked: boolean
) {
  if (key === 'vision') return { ...form, supports_vision: checked };
  if (key === 'function_calling') return { ...form, supports_function_calling: checked };
  if (key === 'streaming') return { ...form, supports_streaming: checked };
  if (key === 'extended_thinking') return { ...form, supports_extended_thinking: checked };
  if (key === 'structured_output') return { ...form, supports_structured_output: checked };
  if (key === 'temperature') return { ...form, supports_temperature: checked };
  if (key === 'attachment') return { ...form, supports_attachment: checked };
  return { ...form, open_weights: checked };
}
