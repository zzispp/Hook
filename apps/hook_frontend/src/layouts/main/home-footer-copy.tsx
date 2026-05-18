'use client';

import Box from '@mui/material/Box';

import { useTranslate } from 'src/locales/use-locales';

// ----------------------------------------------------------------------

export function HomeFooterCopy() {
  const { t } = useTranslate('common');

  return <Box sx={{ mt: 1, typography: 'caption' }}>{t('home.footer.copy')}</Box>;
}
