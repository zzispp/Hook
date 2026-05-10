import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { TranslationManagementView } from 'src/sections/admin/translation-management-view';

export const metadata: Metadata = {
  title: `翻译管理 | Dashboard - ${CONFIG.appName}`,
};

export default function Page() {
  return <TranslationManagementView />;
}
