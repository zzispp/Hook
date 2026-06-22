'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import { useState, useCallback } from 'react';
import { useCopyToClipboard } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Collapse from '@mui/material/Collapse';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

const TOKEN_SCRIPT = `(async () => {
  const authData = {
    refresh_token: localStorage.getItem('refresh_token'),
    token_expires_at: localStorage.getItem('token_expires_at'),
    auth_token: localStorage.getItem('auth_token')
  };

  const jsonOutput = JSON.stringify(authData, null, 2);
  console.log(jsonOutput);

  try {
    if (typeof copy === 'function') {
      copy(jsonOutput);
      console.log('Copied JSON to clipboard via DevTools copy().');
      return;
    }

    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(jsonOutput);
      console.log('Copied JSON to clipboard.');
      return;
    }

    console.log('Clipboard API is not available. Copy the JSON output manually.');
  } catch (error) {
    console.warn('Auto-copy failed. Use the logged JSON manually.', error);
  }
})();`;

type ParsedTokenPayload = {
  authToken: string;
  refreshToken: string;
  tokenExpiresAt: string;
};

export function ProviderQuickImportSub2apiTokenHelp({
  disabled = false,
  onApply,
  sx,
}: {
  disabled?: boolean;
  onApply: (payload: ParsedTokenPayload) => void;
  sx?: SxProps<Theme>;
}) {
  const { t } = useTranslate('admin');
  const { copy } = useCopyToClipboard();
  const [expanded, setExpanded] = useState(false);
  const [jsonInput, setJsonInput] = useState('');

  const handleCopyScript = useCallback(() => {
    copy(TOKEN_SCRIPT);
    toast.success(t('common.copied'));
  }, [copy, t]);

  const handleApplyJson = useCallback(() => {
    try {
      const parsed = JSON.parse(jsonInput) as {
        auth_token?: unknown;
        refresh_token?: unknown;
        token_expires_at?: unknown;
      };
      const authToken = String(parsed.auth_token ?? '').trim();
      const refreshToken = String(parsed.refresh_token ?? '').trim();
      const tokenExpiresAt = String(parsed.token_expires_at ?? '').trim();

      if (!authToken || !refreshToken || !tokenExpiresAt) {
        throw new Error('missing_fields');
      }

      onApply({ authToken, refreshToken, tokenExpiresAt });
      toast.success(t('providers.quickImportSub2apiTokenJsonApplied'));
    } catch {
      toast.error(t('providers.quickImportSub2apiTokenJsonInvalid'));
    }
  }, [jsonInput, onApply, t]);

  return (
    <Alert severity="warning" variant="outlined" sx={sx}>
      <Stack spacing={1.25}>
        <Typography variant="body2">{t('providers.quickImportSub2apiTokenWarning')}</Typography>
        <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between">
          <Stack direction="row" spacing={1} alignItems="center">
            <Button
              size="small"
              color="inherit"
              endIcon={<Iconify width={16} icon={expanded ? 'eva:arrow-upward-fill' : 'eva:arrow-downward-fill'} />}
              onClick={() => setExpanded((current) => !current)}
              sx={{ alignSelf: 'flex-start' }}
            >
              {t('providers.quickImportSub2apiTokenScriptToggle')}
            </Button>
            <Button
              size="small"
              color="inherit"
              disabled={disabled}
              startIcon={<Iconify width={16} icon="solar:copy-bold" />}
              onClick={handleCopyScript}
            >
              {t('providers.quickImportSub2apiTokenCopyScript')}
            </Button>
          </Stack>
        </Stack>
        <Collapse in={expanded} timeout="auto" unmountOnExit>
          <Box
            component="pre"
            sx={{
              m: 0,
              px: 1.5,
              py: 1.25,
              borderRadius: 1,
              bgcolor: 'background.neutral',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
              typography: 'caption',
            }}
          >
            {TOKEN_SCRIPT}
          </Box>
        </Collapse>
        <Stack spacing={1}>
          <TextField
            disabled={disabled}
            multiline
            minRows={4}
            label={t('providers.quickImportSub2apiTokenJsonInput')}
            placeholder={t('providers.quickImportSub2apiTokenJsonPlaceholder')}
            value={jsonInput}
            onChange={(event) => setJsonInput(event.target.value)}
          />
          <Stack direction="row" justifyContent="flex-end">
            <Button disabled={disabled || !jsonInput.trim()} variant="contained" size="small" onClick={handleApplyJson}>
              {t('providers.quickImportSub2apiTokenApplyJson')}
            </Button>
          </Stack>
        </Stack>
      </Stack>
    </Alert>
  );
}
