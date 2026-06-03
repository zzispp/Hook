import { useContext } from 'react';

import {
  InstallationContext,
  type InstallationContextValue,
} from '../components/context/InstallationContext/InstallationContext';

export const useInstallation = (): InstallationContextValue => {
  const ctx = useContext(InstallationContext);
  if (!ctx) throw new Error('useInstallation must be used within InstallationProvider');
  return ctx;
};
