import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { EditorView } from 'src/sections/_examples/extra/editor-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Editor | Components - ${CONFIG.appName}` };

export default function Page() {
  return <EditorView />;
}
