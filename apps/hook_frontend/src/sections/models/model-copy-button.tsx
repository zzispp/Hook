'use client';

import { useCallback } from 'react';

import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { copyText } from 'src/utils/browser-compat';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type Props = {
  value: string;
};

export function ModelCopyButton({ value }: Props) {
  const { t } = useTranslate('admin');
  const handleCopy = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation();
      void copyModelId(value, t);
    },
    [t, value]
  );

  return (
    <Tooltip title={t('models.copyModelId')}>
      <IconButton size="small" onClick={handleCopy}>
        <Iconify width={16} icon="solar:copy-bold" />
      </IconButton>
    </Tooltip>
  );
}

async function copyModelId(value: string, t: (key: string) => string) {
  try {
    await copyText(value);
    toast.success(t('models.modelIdCopied'));
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.copyFailed'));
  }
}
