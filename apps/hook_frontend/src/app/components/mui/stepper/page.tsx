import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { StepperView } from 'src/sections/_examples/mui/stepper-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Stepper | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <StepperView />;
}
