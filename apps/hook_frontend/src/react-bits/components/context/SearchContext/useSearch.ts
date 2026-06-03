import { useContext } from 'react';

import { SearchContext, type SearchContextValue } from './SearchContext';

export function useSearch(): SearchContextValue {
  const ctx = useContext(SearchContext);
  if (!ctx) throw new Error('useSearch must be used within SearchProvider');
  return ctx;
}
