import { useContext } from 'react';

import { OptionsContext, type OptionsContextValue } from './OptionsContext';

export function useOptions(): OptionsContextValue {
  const ctx = useContext(OptionsContext);
  if (!ctx) throw new Error('useOptions must be used within OptionsProvider');
  return ctx;
}
