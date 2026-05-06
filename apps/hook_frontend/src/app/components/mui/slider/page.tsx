import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { SliderView } from 'src/sections/_examples/mui/slider-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Slider | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <SliderView />;
}
