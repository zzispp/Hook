'use client';

import type { CSSProperties } from 'react';

import Script from 'next/script';
import { useRef, useMemo, useEffect, createElement } from 'react';

import Box from '@mui/material/Box';

import { endpoints } from 'src/lib/axios';
import { CONFIG } from 'src/global-config';

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
  const widgetRef = useRef<CapElement | null>(null);
  const apiEndpoint = useMemo(() => captchaApiEndpoint(), []);
  const widgetStyle = useMemo(
    () =>
      ({
        display: 'block',
        width: '100%',
        '--cap-widget-width': '100%',
      }) as CSSProperties,
    []
  );

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
        'data-cap-i18n-initial-state': "Verify you're human",
        'data-cap-i18n-verifying-label': 'Verifying...',
        'data-cap-i18n-solved-label': 'Verified',
        'data-cap-i18n-error-label': 'Verification failed. Try again.',
      })}
    </Box>
  );
}

function captchaApiEndpoint() {
  const baseUrl = CONFIG.serverUrl.replace(/\/$/, '');
  return `${baseUrl}${endpoints.captcha.apiEndpoint}`;
}
