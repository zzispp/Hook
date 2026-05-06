import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { CarouselView } from 'src/sections/_examples/extra/carousel-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Carousel | Components - ${CONFIG.appName}` };

export default function Page() {
  return <CarouselView />;
}
