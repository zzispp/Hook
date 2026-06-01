import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/locales/use-locales';

import { Field } from 'src/components/hook-form';

export function ProfileEmailCodeFields({
  sendingCode,
  isSubmitting,
  onSendCode,
}: {
  sendingCode: boolean;
  isSubmitting: boolean;
  onSendCode: VoidFunction;
}) {
  const { t } = useTranslate('common');

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        disabled
        name="email"
        label={t('profile.email')}
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Button
        fullWidth
        size="large"
        type="button"
        color="inherit"
        variant="outlined"
        loading={sendingCode}
        disabled={isSubmitting}
        loadingIndicator={t('profile.sendCodeLoading')}
        onClick={onSendCode}
      >
        {t('profile.sendCode')}
      </Button>

      <Field.Code name="code" />
    </Box>
  );
}
