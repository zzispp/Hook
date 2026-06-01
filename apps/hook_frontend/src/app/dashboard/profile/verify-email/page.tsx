import type { Metadata } from 'next';

import { dashboardPageMetadata } from 'src/app/dashboard/page-metadata';
import { DASHBOARD_MENU_CODES } from 'src/layouts/dashboard/dashboard-menu-values';

import { ProfileVerifyEmailView } from 'src/sections/profile/profile-verify-email-view';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  return dashboardPageMetadata(DASHBOARD_MENU_CODES.profile);
}

export default function Page() {
  return <ProfileVerifyEmailView />;
}
