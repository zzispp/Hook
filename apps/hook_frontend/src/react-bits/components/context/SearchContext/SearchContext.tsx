'use client';

import type { ReactNode } from 'react';

import { useState, createContext } from 'react';

export type SearchContextValue = {
  readonly isSearchOpen: boolean;
  readonly openSearch: () => void;
  readonly closeSearch: () => void;
  readonly toggleSearch: () => void;
};

export const SearchContext = createContext<SearchContextValue | null>(null);

export function SearchProvider({ children }: { readonly children: ReactNode }) {
  const [isSearchOpen, setSearchOpen] = useState(false);

  const openSearch = () => setSearchOpen(true);
  const closeSearch = () => setSearchOpen(false);
  const toggleSearch = () => setSearchOpen((open) => !open);

  return (
    <SearchContext.Provider value={{ openSearch, closeSearch, toggleSearch, isSearchOpen }}>
      {children}
    </SearchContext.Provider>
  );
}
