import type { NextRequest } from 'next/server';

import { kebabCase } from 'es-toolkit';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _posts } from 'src/_mock/_blog';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * Get latest posts
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const title = searchParams.get('title');

    const posts = _posts();

    const latestPosts = posts.filter((_post) => kebabCase(_post.title) !== title);

    logger('[Post] latest-list', latestPosts.length);

    return response({ latestPosts }, STATUS.OK);
  } catch (error) {
    return handleError('Post - Get latest', error);
  }
}
