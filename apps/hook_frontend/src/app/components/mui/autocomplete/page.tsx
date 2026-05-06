import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { AutocompleteView } from 'src/sections/_examples/mui/autocomplete-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Autocomplete | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <AutocompleteView />;
}
