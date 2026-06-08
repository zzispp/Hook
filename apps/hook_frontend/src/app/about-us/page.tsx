import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { AboutView } from 'src/sections/about/view';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('aboutUs');
}

export default function Page() {
  return <AboutView />;
}
