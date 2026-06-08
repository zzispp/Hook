import type { Metadata } from 'next';

import { commonPageMetadata } from 'src/app/page-metadata';

import { View403 } from 'src/sections/error';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return commonPageMetadata('page403');
}

export default function Page() {
  return <View403 />;
}
