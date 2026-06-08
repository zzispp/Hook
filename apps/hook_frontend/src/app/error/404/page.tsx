import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { NotFoundView } from 'src/sections/error';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('page404');
}

export default function Page() {
  return <NotFoundView />;
}
