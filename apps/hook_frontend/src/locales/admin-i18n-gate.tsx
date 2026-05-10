'use client';

import { useState, useEffect } from 'react';

import Alert from '@mui/material/Alert';
import Container from '@mui/material/Container';

import { useAdminI18nResource } from 'src/actions/i18n';

import { LoadingScreen } from 'src/components/loading-screen';

import { useTranslate } from './use-locales';

type Props = {
  children: React.ReactNode;
};

const ADMIN_NAMESPACE = 'admin';

export function AdminI18nGate({ children }: Props) {
  const { i18n, currentLang } = useTranslate();
  const resource = useAdminI18nResource(currentLang.value, ADMIN_NAMESPACE);
  const [loadedKey, setLoadedKey] = useState('');

  useEffect(() => {
    if (!resource.data) {
      return;
    }
    i18n.addResourceBundle(
      resource.data.lang,
      resource.data.namespace,
      resource.data.resources,
      true,
      true
    );
    setLoadedKey(resourceKey(resource.data.lang, resource.data.namespace));
  }, [i18n, resource.data]);

  if (resource.error) {
    return (
      <Container sx={{ py: 10 }}>
        <Alert severity="error">{errorMessage(resource.error)}</Alert>
      </Container>
    );
  }

  if (resource.isLoading || !resource.data || loadedKey !== resourceKey(currentLang.value, ADMIN_NAMESPACE)) {
    return <LoadingScreen portal={false} />;
  }

  return <>{children}</>;
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Failed to load admin translations.';
}

function resourceKey(lang: string, namespace: string) {
  return `${lang}:${namespace}`;
}
