'use client';

import { useCallback } from 'react';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

type Props = {
  value: string;
};

export function ModelCopyButton({ value }: Props) {
  const { t } = useTranslate('admin');
  const { copy } = useCopyToClipboard();
  const handleCopy = useCallback(
    (event: React.MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation();
      copy(value);
      toast.success(t('models.modelIdCopied'));
    },
    [copy, t, value]
  );

  return (
    <Tooltip title={t('models.copyModelId')}>
      <IconButton size="small" onClick={handleCopy}>
        <Iconify width={16} icon="solar:copy-bold" />
      </IconButton>
    </Tooltip>
  );
}
