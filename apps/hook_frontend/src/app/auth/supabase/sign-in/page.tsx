import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SupabaseSignInView } from 'src/auth/view/supabase';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | Supabase - ${CONFIG.appName}` };

export default function Page() {
  return <SupabaseSignInView />;
}
