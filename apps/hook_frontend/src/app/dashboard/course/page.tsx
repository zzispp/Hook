import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { OverviewCourseView } from 'src/sections/overview/course/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Course | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <OverviewCourseView />;
}
