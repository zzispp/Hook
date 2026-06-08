import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import cnCommon from 'src/locales/langs/cn/common.json';

import { NotFoundView } from 'src/sections/error';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: `${cnCommon.metadata.pages.page404} - ${CONFIG.appName}`,
};

export default function Page() {
  return <NotFoundView />;
}
