'use client';

import type { IconifyName } from 'src/components/iconify';
import type { SystemSettingsForm } from './system-settings-utils';
import type { ContactMethod, ContactMethodType } from 'src/types/system-setting';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { TextFieldRow, NAV_ICON_OPTIONS } from './shared';
import { SettingsSection } from './system-settings-section';
import {
  addContactMethod,
  moveContactMethod,
  handleQrCodeUpload,
  removeContactMethod,
  updateContactMethod,
  CONTACT_ICON_BY_TYPE,
  CONTACT_TYPE_OPTIONS,
} from './system-settings-contact-methods-utils';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function SystemSettingsContactMethodsSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection
      title={t('systemSettings.sections.contactMethods')}
      description={t('systemSettings.contactMethods.description')}
    >
      <Stack spacing={2}>
        {form.contact_methods.map((method, index) => (
          <ContactMethodEditor
            key={method.id}
            method={method}
            index={index}
            total={form.contact_methods.length}
            onChange={(next) => updateContactMethod(setForm, index, next)}
            onMove={(direction) => moveContactMethod(setForm, index, direction)}
            onRemove={() => removeContactMethod(setForm, index)}
          />
        ))}
        <Button
          variant="outlined"
          startIcon={<Iconify icon="mingcute:add-line" />}
          onClick={() => addContactMethod(setForm)}
        >
          {t('systemSettings.contactMethods.add')}
        </Button>
      </Stack>
    </SettingsSection>
  );
}

function ContactMethodEditor({
  method,
  index,
  total,
  onChange,
  onMove,
  onRemove,
}: {
  method: ContactMethod;
  index: number;
  total: number;
  onChange: (method: ContactMethod) => void;
  onMove: (direction: -1 | 1) => void;
  onRemove: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={2} sx={{ p: 2, border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
      <ContactMethodHeader index={index} total={total} onMove={onMove} onRemove={onRemove} />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <ContactTypeField method={method} onChange={onChange} />
        {method.type === 'custom' ? (
          <TextFieldRow
            required
            label={t('systemSettings.fields.contactCustomType')}
            value={method.custom_type}
            onChange={(value) => onChange({ ...method, custom_type: value })}
          />
        ) : null}
      </Stack>
      <ContactIconField method={method} onChange={onChange} />
      <TextFieldRow
        required
        label={t('systemSettings.fields.contactValue')}
        value={method.value}
        onChange={(value) => onChange({ ...method, value })}
      />
      <ContactQrCodeField method={method} onChange={onChange} />
    </Stack>
  );
}

function ContactMethodHeader({
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
        {t('systemSettings.contactMethods.itemTitle', { index: index + 1 })}
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

function ContactTypeField({
  method,
  onChange,
}: {
  method: ContactMethod;
  onChange: (method: ContactMethod) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      required
      select
      label={t('systemSettings.fields.contactType')}
      value={method.type}
      onChange={(value) => {
        const type = value as ContactMethodType;
        onChange({ ...method, type, icon: CONTACT_ICON_BY_TYPE[type] });
      }}
    >
      {CONTACT_TYPE_OPTIONS.map((type) => (
        <MenuItem key={type} value={type}>
          {t(`systemSettings.contactMethods.types.${type}`)}
        </MenuItem>
      ))}
    </TextFieldRow>
  );
}

function ContactIconField({
  method,
  onChange,
}: {
  method: ContactMethod;
  onChange: (method: ContactMethod) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Autocomplete
      fullWidth
      options={NAV_ICON_OPTIONS}
      value={method.icon || null}
      onChange={(_event, value) => onChange({ ...method, icon: value || '' })}
      renderOption={(props, option) => (
        <MenuItem {...props} key={option} value={option}>
          <Stack direction="row" spacing={1.25} alignItems="center">
            <Iconify icon={option as IconifyName} />
            <Typography variant="body2">{option}</Typography>
          </Stack>
        </MenuItem>
      )}
      renderInput={(params) => (
        <TextField
          {...params}
          required
          label={t('systemSettings.fields.contactIcon')}
          slotProps={{
            input: {
              ...params.InputProps,
              startAdornment: method.icon ? (
                <Iconify icon={method.icon as IconifyName} sx={{ mr: 1, color: 'text.secondary' }} />
              ) : (
                params.InputProps.startAdornment
              ),
            },
          }}
        />
      )}
    />
  );
}

function ContactQrCodeField({
  method,
  onChange,
}: {
  method: ContactMethod;
  onChange: (method: ContactMethod) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Stack spacing={1.5}>
      <TextFieldRow
        label={t('systemSettings.fields.contactQrCode')}
        value={method.qr_code}
        helperText={t('systemSettings.helper.contactQrCode')}
        onChange={(value) => onChange({ ...method, qr_code: value })}
      />
      <Stack direction="row" spacing={1} alignItems="center">
        <Button component="label" variant="outlined" startIcon={<Iconify icon="solar:import-bold" />}>
          {t('systemSettings.contactMethods.uploadQrCode')}
          <Box
            component="input"
            hidden
            type="file"
            accept="image/*"
            onChange={(event) => handleQrCodeUpload(event, method, onChange)}
          />
        </Button>
        {method.qr_code ? (
          <Button color="inherit" onClick={() => onChange({ ...method, qr_code: '' })}>
            {t('common.clear')}
          </Button>
        ) : null}
      </Stack>
      {method.qr_code ? <QrCodePreview src={method.qr_code} /> : null}
    </Stack>
  );
}

function QrCodePreview({ src }: { src: string }) {
  return (
    <Box
      component="img"
      src={src}
      alt=""
      sx={{ width: 96, height: 96, objectFit: 'cover', borderRadius: 1, border: '1px solid', borderColor: 'divider' }}
    />
  );
}
