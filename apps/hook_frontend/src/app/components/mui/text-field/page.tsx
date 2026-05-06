import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TextFieldView } from 'src/sections/_examples/mui/text-field-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Text field | MUI - ${CONFIG.appName}` };

export default function Page() {
  return <TextFieldView />;
}
