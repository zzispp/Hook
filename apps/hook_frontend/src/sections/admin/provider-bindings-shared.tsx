'use client';

import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

export function EmptyList({ loading, length }: { loading: boolean; length: number }) {
  const { t } = useTranslate('admin');
  if (loading) {
    return (
      <Typography variant="body2" color="text.secondary" sx={{ p: 2 }}>
        {t('common.loading')}
      </Typography>
    );
  }
  if (length > 0) return null;
  return (
    <Typography variant="body2" color="text.secondary" sx={{ p: 2 }}>
      {t('common.noData')}
    </Typography>
  );
}
