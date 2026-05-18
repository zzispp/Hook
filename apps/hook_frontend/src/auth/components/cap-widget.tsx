'use client';

import type { CSSProperties } from 'react';
import type { Theme } from '@mui/material/styles';

import Script from 'next/script';
import { varAlpha } from 'minimal-shared/utils';
import { useRef, useMemo, useEffect, createElement } from 'react';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';

import { endpoints } from 'src/lib/axios';
import { CONFIG } from 'src/global-config';
import { useTranslate } from 'src/locales/use-locales';

type CapElement = HTMLElement & {
  reset?: () => void;
  token?: string | null;
};

type CapSolveEvent = CustomEvent<{ token: string }>;

type CapWidgetProps = {
  enabled: boolean;
  resetKey: number;
  onTokenChange: (token: string | null) => void;
};

export function AuthCaptcha({ enabled, resetKey, onTokenChange }: CapWidgetProps) {
  const theme = useTheme();
  const { t } = useTranslate('auth');
  const widgetRef = useRef<CapElement | null>(null);
  const apiEndpoint = useMemo(() => captchaApiEndpoint(), []);
  const widgetStyle = useMemo(() => capWidgetStyle(theme), [theme]);

  useEffect(() => {
    const widget = widgetRef.current;
    if (!enabled || !widget) {
      onTokenChange(null);
      return undefined;
    }

    const handleSolve = (event: Event) => {
      const token = (event as CapSolveEvent).detail.token;
      onTokenChange(token || null);
    };
    const handleReset = () => onTokenChange(null);
    const handleError = () => onTokenChange(null);

    widget.addEventListener('solve', handleSolve);
    widget.addEventListener('reset', handleReset);
    widget.addEventListener('error', handleError);

    return () => {
      widget.removeEventListener('solve', handleSolve);
      widget.removeEventListener('reset', handleReset);
      widget.removeEventListener('error', handleError);
    };
  }, [enabled, onTokenChange]);

  useEffect(() => {
    const widget = widgetRef.current;
    if (!enabled || !widget) {
      return;
    }
    widget.reset?.();
    onTokenChange(null);
  }, [enabled, onTokenChange, resetKey]);

  if (!enabled) {
    return null;
  }

  return (
    <Box sx={{ width: 1 }}>
      <Script src="/assets/js/cap.min.js" strategy="afterInteractive" />
      {createElement('cap-widget', {
        ref: widgetRef,
        required: true,
        style: widgetStyle,
        'data-cap-api-endpoint': apiEndpoint,
        'data-cap-hidden-field-name': 'captcha_token',
        'data-cap-i18n-initial-state': t('captcha.initial'),
        'data-cap-i18n-verifying-label': t('captcha.verifying'),
        'data-cap-i18n-solved-label': t('captcha.solved'),
        'data-cap-i18n-error-label': t('captcha.error'),
      })}
    </Box>
  );
}

function captchaApiEndpoint() {
  const baseUrl = CONFIG.serverUrl.replace(/\/$/, '');
  return `${baseUrl}${endpoints.captcha.apiEndpoint}`;
}

function capWidgetStyle(theme: Theme) {
  const palette = theme.vars.palette;

  return {
    display: 'block',
    width: '100%',
    colorScheme: theme.palette.mode,
    '--cap-widget-width': '100%',
    '--cap-background': palette.background.paper,
    '--cap-color': palette.text.primary,
    '--cap-border-color': varAlpha(palette.grey['500Channel'], 0.2),
    '--cap-focus-ring': palette.primary.main,
    '--cap-checkbox-border': `1px solid ${varAlpha(palette.grey['500Channel'], 0.32)}`,
    '--cap-checkbox-background': varAlpha(palette.grey['500Channel'], 0.08),
    '--cap-spinner-background-color': varAlpha(palette.grey['500Channel'], 0.16),
    '--cap-spinner-color': palette.primary.main,
    '--cap-invalid-border-color': palette.error.main,
    '--cap-invalid-ring-color': varAlpha(palette.error.mainChannel, 0.2),
    '--cap-troubleshoot-color': palette.primary.main,
  } as CSSProperties;
}
