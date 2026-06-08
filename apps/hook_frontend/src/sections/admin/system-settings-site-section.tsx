'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { logoImageSource, isMaskableLogoSource } from 'src/components/logo/logo-utils';

import { TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function SystemSettingsSiteSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');

  return (
    <SettingsSection title={t('systemSettings.sections.site')}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          required
          label={t('systemSettings.fields.siteName')}
          value={form.site_name}
          onChange={(value) => setForm((current) => ({ ...current, site_name: value }))}
        />
        <TextFieldRow
          label={t('systemSettings.fields.siteSubtitle')}
          value={form.site_subtitle}
          onChange={(value) => setForm((current) => ({ ...current, site_subtitle: value }))}
        />
      </Stack>
      <TextFieldRow
        label={t('systemSettings.fields.publicBaseUrl')}
        value={form.public_base_url}
        onChange={(value) => setForm((current) => ({ ...current, public_base_url: value }))}
        sx={{ mt: 2 }}
      />
      <LogoField form={form} setForm={setForm} />
    </SettingsSection>
  );
}

function LogoField({ form, setForm }: Props) {
  const { t } = useTranslate('admin');
  const logoSrc = logoImageSource(form.site_logo_base64);

  return (
    <Stack
      spacing={2}
      direction={{ xs: 'column', md: 'row' }}
      sx={{ mt: 2, alignItems: 'center' }}
    >
      <Box
        sx={{
          p: 1,
          width: 64,
          height: 64,
          display: 'flex',
          borderRadius: 1,
          overflow: 'hidden',
          alignItems: 'center',
          border: '1px solid',
          borderColor: 'divider',
          justifyContent: 'center',
        }}
      >
        {logoSrc && isMaskableLogoSource(logoSrc) && (
          <Box
            aria-hidden
            component="span"
            sx={{
              width: 1,
              height: 1,
              color: 'text.primary',
              display: 'block',
              bgcolor: 'currentColor',
              mask: `url("${logoSrc}") center / contain no-repeat`,
              WebkitMask: `url("${logoSrc}") center / contain no-repeat`,
            }}
          />
        )}
        {logoSrc && !isMaskableLogoSource(logoSrc) && (
          <Box
            component="img"
            alt=""
            src={logoSrc}
            sx={{ width: 1, height: 1, display: 'block', objectFit: 'contain' }}
          />
        )}
      </Box>
      <Button component="label" variant="outlined" startIcon={<Iconify icon="solar:import-bold" />}>
        {t('systemSettings.fields.siteLogoUpload')}
        <input hidden type="file" accept="image/*" onChange={(event) => readLogoFile(event, setForm)} />
      </Button>
    </Stack>
  );
}

function readLogoFile(
  event: React.ChangeEvent<HTMLInputElement>,
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>
) {
  const file = event.target.files?.[0];
  event.target.value = '';
  if (!file) {
    return;
  }

  const reader = new FileReader();
  reader.onload = () => {
    const result = reader.result;
    if (typeof result !== 'string') {
      return;
    }
    setForm((current) => ({ ...current, site_logo_base64: result }));
  };
  reader.readAsDataURL(file);
}
