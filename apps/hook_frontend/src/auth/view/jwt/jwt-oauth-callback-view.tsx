'use client';

import type { OAuthProvider, OAuthCallbackContentProps } from './jwt-oauth-callback-controller';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';

import {
  ACCOUNT_OAUTH_BINDING_KEY,
  useOAuthCallbackController,
} from './jwt-oauth-callback-controller';

export { ACCOUNT_OAUTH_BINDING_KEY };

type Props = {
  provider: OAuthProvider;
};

export function JwtOAuthCallbackView({ provider }: Props) {
  return <OAuthCallbackContent {...useOAuthCallbackController(provider)} />;
}

function OAuthCallbackContent(props: OAuthCallbackContentProps) {
  const { t } = useTranslate('auth');
  const { binding, errorMessage, loading, submitting, onCancelBinding, onConfirmBinding } = props;

  return (
    <Box sx={{ textAlign: 'center' }}>
      {loading && (
        <Box sx={{ py: 4 }}>
          <CircularProgress />
          <Typography variant="body2" sx={{ mt: 2, color: 'text.secondary' }}>
            {t('social.oauthProcessing')}
          </Typography>
        </Box>
      )}

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
          {errorMessage}
        </Alert>
      )}

      {binding && (
        <Box sx={{ gap: 2, display: 'flex', flexDirection: 'column' }}>
          <Typography variant="h5">{t('social.bindingRequiredTitle')}</Typography>
          <Typography variant="body2" color="text.secondary">
            {t('social.bindingRequiredDescription', {
              provider: providerLabel(binding.provider),
              email: binding.email,
              username: binding.username,
            })}
          </Typography>
          <Button
            color="inherit"
            variant="contained"
            loading={submitting}
            onClick={onConfirmBinding}
          >
            {t('social.confirmBinding')}
          </Button>
          <Button color="inherit" variant="text" onClick={onCancelBinding}>
            {t('social.cancelBinding')}
          </Button>
        </Box>
      )}
    </Box>
  );
}

function providerLabel(provider: OAuthProvider) {
  return provider === 'github' ? 'GitHub' : 'Google';
}
