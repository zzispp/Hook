'use client';

import type { Namespace } from 'i18next';

import { useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

import { useSettingsContext } from 'src/components/settings';

import { getCurrentLang } from './locales-config';

// ----------------------------------------------------------------------

export function useTranslate(namespace?: Namespace) {
  const { t, i18n } = useTranslation(namespace);
  const currentLang = getCurrentLang(i18n.resolvedLanguage);

  return {
    t,
    i18n,
    currentLang,
  };
}

// ----------------------------------------------------------------------

export function useLocaleDirectionSync() {
  const { i18n, currentLang } = useTranslate();
  const { state, setState } = useSettingsContext();

  const handleSync = useCallback(async () => {
    const selectedLang = currentLang.value;
    const i18nDir = i18n.dir(selectedLang);

    if (document.dir !== i18nDir) {
      document.dir = i18nDir;
    }

    if (state.direction !== i18nDir) {
      setState({ direction: i18nDir });
    }

    if (i18n.resolvedLanguage !== selectedLang) {
      await i18n.changeLanguage(selectedLang);
    }
  }, [currentLang.value, i18n, setState, state.direction]);

  useEffect(() => {
    handleSync();
  }, [handleSync]);

  return null;
}
