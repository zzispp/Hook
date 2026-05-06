import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';
import { getPosts } from 'src/actions/blog-ssr';

import { PostListHomeView } from 'src/sections/blog/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Post list - ${CONFIG.appName}` };

export default async function Page() {
  const { posts } = await getPosts();

  return <PostListHomeView posts={posts} />;
}
