import type { Metadata } from 'next';
import type { IPostItem } from 'src/types/blog';

import { kebabCase } from 'es-toolkit';

import { CONFIG } from 'src/global-config';
import { getPost } from 'src/actions/blog-ssr';
import axios, { endpoints } from 'src/lib/axios';

import { PostDetailsView } from 'src/sections/blog/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Post details | Dashboard - ${CONFIG.appName}` };

type Props = {
  params: Promise<{ title: string }>;
};

export default async function Page({ params }: Props) {
  const { title } = await params;

  const { post } = await getPost(title);

  return <PostDetailsView post={post} />;
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
  const res = await axios.get(endpoints.post.list);
  const data: IPostItem[] = CONFIG.isStaticExport ? res.data.posts : res.data.posts.slice(0, 1);

  return data.map((post) => ({
    title: kebabCase(post.title),
  }));
}
