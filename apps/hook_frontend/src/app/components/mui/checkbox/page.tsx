import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CheckboxView } from 'src/sections/_examples/mui/checkbox-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Checkbox | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <CheckboxView />;
}
