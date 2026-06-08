import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { View500 } from 'src/sections/error';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('page500');
}

export default function Page() {
  return <View500 />;
}
