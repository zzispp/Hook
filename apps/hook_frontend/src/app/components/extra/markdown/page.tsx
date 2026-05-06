import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { MarkdownView } from 'src/sections/_examples/extra/markdown-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Markdown | Components - ${CONFIG.appName}` };

export default function Page() {
  return <MarkdownView />;
}
