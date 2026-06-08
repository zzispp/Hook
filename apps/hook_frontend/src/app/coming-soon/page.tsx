import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { ComingSoonView } from 'src/sections/coming-soon/view';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('comingSoon');
}

export default function Page() {
  return <ComingSoonView />;
}
