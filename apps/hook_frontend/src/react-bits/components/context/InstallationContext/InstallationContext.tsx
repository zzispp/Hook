'use client';

import type { Dispatch, ReactNode, SetStateAction } from 'react';

import { useState, useEffect, useCallback, createContext } from 'react';

export type InstallMode = 'manual';
export type CliTool = 'shadcn' | 'jsrepo';
export type PackageManager = 'npm' | 'pnpm' | 'bun' | 'yarn';

type StorageKey = 'rb_install_mode' | 'rb_cli_tool' | 'rb_pkg_manager';

export type InstallationContextValue = {
  readonly installMode: InstallMode;
  readonly setInstallMode: Dispatch<SetStateAction<InstallMode>>;
  readonly cliTool: CliTool;
  readonly setCliTool: Dispatch<SetStateAction<CliTool>>;
  readonly packageManager: PackageManager;
  readonly setPackageManager: Dispatch<SetStateAction<PackageManager>>;
};

export const InstallationContext = createContext<InstallationContextValue | null>(null);

const isInstallMode = (value: string | null): value is InstallMode => value === 'manual';

const isCliTool = (value: string | null): value is CliTool =>
  value === 'shadcn' || value === 'jsrepo';

const isPackageManager = (value: string | null): value is PackageManager =>
  value === 'npm' || value === 'pnpm' || value === 'bun' || value === 'yarn';

function readStoredValue<T extends string>(
  key: StorageKey,
  fallback: T,
  isAllowed: (value: string | null) => value is T
): T {
  const storedValue = localStorage.getItem(key);
  return isAllowed(storedValue) ? storedValue : fallback;
}

export function InstallationProvider({ children }: { readonly children: ReactNode }) {
  const [installMode, setInstallMode] = useState<InstallMode>('manual');
  const [cliTool, setCliTool] = useState<CliTool>('shadcn');
  const [packageManager, setPackageManager] = useState<PackageManager>('npm');

  useEffect(() => {
    setInstallMode(readStoredValue('rb_install_mode', 'manual', isInstallMode));
    setCliTool(readStoredValue('rb_cli_tool', 'shadcn', isCliTool));
    setPackageManager(readStoredValue('rb_pkg_manager', 'npm', isPackageManager));
  }, []);

  useEffect(() => {
    localStorage.setItem('rb_install_mode', installMode);
  }, [installMode]);

  useEffect(() => {
    localStorage.setItem('rb_cli_tool', cliTool);
  }, [cliTool]);

  useEffect(() => {
    localStorage.setItem('rb_pkg_manager', packageManager);
  }, [packageManager]);

  const value = {
    installMode,
    setInstallMode: useCallback((mode: SetStateAction<InstallMode>) => setInstallMode(mode), []),
    cliTool,
    setCliTool: useCallback((tool: SetStateAction<CliTool>) => setCliTool(tool), []),
    packageManager,
    setPackageManager: useCallback(
      (manager: SetStateAction<PackageManager>) => setPackageManager(manager),
      []
    ),
  };

  return <InstallationContext.Provider value={value}>{children}</InstallationContext.Provider>;
}
