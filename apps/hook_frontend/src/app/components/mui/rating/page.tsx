import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { RatingView } from 'src/sections/_examples/mui/rating-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Rating | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <RatingView />;
}
