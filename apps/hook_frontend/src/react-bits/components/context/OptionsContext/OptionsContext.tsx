'use client';

import type { Dispatch, ReactNode, SetStateAction } from 'react';

import { useState, useEffect, useCallback, createContext } from 'react';

export type LanguagePreset = 'JS' | 'TS';
export type StylePreset = 'CSS' | 'TW';

export type OptionsContextValue = {
  readonly languagePreset: LanguagePreset;
  readonly setLanguagePreset: Dispatch<SetStateAction<LanguagePreset>>;
  readonly stylePreset: StylePreset;
  readonly setStylePreset: Dispatch<SetStateAction<StylePreset>>;
  readonly toggleLanguage: () => void;
  readonly toggleStyle: () => void;
};

export const OptionsContext = createContext<OptionsContextValue | null>(null);

export function OptionsProvider({ children }: { readonly children: ReactNode }) {
  const [languagePreset, setLanguagePreset] = useState<LanguagePreset>('JS');
  const [stylePreset, setStylePreset] = useState<StylePreset>('CSS');

  useEffect(() => {
    const storedLang = localStorage.getItem('preferredLanguage');
    const storedStyle = localStorage.getItem('preferredStyle');

    if (storedLang === 'JS' || storedLang === 'TS') setLanguagePreset(storedLang);
    if (storedStyle === 'CSS' || storedStyle === 'TW') setStylePreset(storedStyle);
  }, []);

  useEffect(() => {
    localStorage.setItem('preferredLanguage', languagePreset);
  }, [languagePreset]);

  useEffect(() => {
    localStorage.setItem('preferredStyle', stylePreset);
  }, [stylePreset]);

  const toggleLanguage = useCallback(() => {
    setLanguagePreset((preset) => (preset === 'JS' ? 'TS' : 'JS'));
  }, []);

  const toggleStyle = useCallback(() => {
    setStylePreset((preset) => (preset === 'CSS' ? 'TW' : 'CSS'));
  }, []);

  return (
    <OptionsContext.Provider
      value={{ languagePreset, setLanguagePreset, stylePreset, setStylePreset, toggleLanguage, toggleStyle }}
    >
      {children}
    </OptionsContext.Provider>
  );
}
