import type { Metadata } from 'next';

import { getSiteInfo } from 'src/actions/site-info-server';

import { HomeView } from 'src/sections/home/view';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  const site = await getSiteInfo();

  return {
    title: [site.site_name, site.site_subtitle].filter(Boolean).join(' - '),
    description:
      'Self-hosted AI API gateway for user token management, smart load balancing, cost quota control, and health monitoring.',
  };
}

export default function Page() {
  return <HomeView />;
}
