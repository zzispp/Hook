import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { BlankView } from 'src/sections/blank/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Item with subpaths - ${CONFIG.appName}` };

export default function Page() {
  return (
    <BlankView title="Match subpaths" description="Active on matching path and its subpaths." />
  );
}
