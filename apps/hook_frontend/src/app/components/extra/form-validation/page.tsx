import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { FormValidationView } from 'src/sections/_examples/extra/form-validation-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Form validation | Components - ${CONFIG.appName}` };

export default function Page() {
  return <FormValidationView />;
}
