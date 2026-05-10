import { CONFIG } from 'src/global-config';
import { DashboardLayout } from 'src/layouts/dashboard';
import { AdminI18nGate } from 'src/locales/admin-i18n-gate';

import { AuthGuard } from 'src/auth/guard';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  if (CONFIG.auth.skip) {
    return (
      <AdminI18nGate>
        <DashboardLayout>{children}</DashboardLayout>
      </AdminI18nGate>
    );
  }

  return (
    <AuthGuard>
      <AdminI18nGate>
        <DashboardLayout>{children}</DashboardLayout>
      </AdminI18nGate>
    </AuthGuard>
  );
}
