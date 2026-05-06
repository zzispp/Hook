import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { RadioButtonView } from 'src/sections/_examples/mui/radio-button-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Radio button | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <RadioButtonView />;
}
