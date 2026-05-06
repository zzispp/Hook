import type { Metadata } from 'next';
import type { ITourItem } from 'src/types/tour';

import { _tours } from 'src/_mock/_tour';
import { CONFIG } from 'src/global-config';

import { TourDetailsView } from 'src/sections/tour/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Tour details | Dashboard - ${CONFIG.appName}` };

type Props = {
  params: Promise<{ id: string }>;
};

export default async function Page({ params }: Props) {
  const { id } = await params;

  const currentTour = _tours.find((tour) => tour.id === id);

  return <TourDetailsView tour={currentTour} />;
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
  const data: ITourItem[] = CONFIG.isStaticExport ? _tours : _tours.slice(0, 1);

  return data.map((tour) => ({
    id: tour.id,
  }));
}
