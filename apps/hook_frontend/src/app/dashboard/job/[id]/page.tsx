import type { Metadata } from 'next';
import type { IJobItem } from 'src/types/job';

import { _jobs } from 'src/_mock/_job';
import { CONFIG } from 'src/global-config';

import { JobDetailsView } from 'src/sections/job/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Job details | Dashboard - ${CONFIG.appName}` };

type Props = {
  params: Promise<{ id: string }>;
};

export default async function Page({ params }: Props) {
  const { id } = await params;

  const currentJob = _jobs.find((job) => job.id === id);

  return <JobDetailsView job={currentJob} />;
}

// ----------------------------------------------------------------------

/**
 * Static Exports in Next.js
 *
 * 1. Set `isStaticExport = true` in `next.config.{mjs|ts}`.
 * 2. This allows `generateStaticParams()` to pre-render dynamic routes at build time.
 *
 * For more details, see:
 * https://nextjs.org/docs/app/building-your-application/deploying/static-exports
 *
 * NOTE: Remove all "generateStaticParams()" functions if not using static exports.
 */
export async function generateStaticParams() {
  const data: IJobItem[] = CONFIG.isStaticExport ? _jobs : _jobs.slice(0, 1);

  return data.map((job) => ({
    id: job.id,
  }));
}
