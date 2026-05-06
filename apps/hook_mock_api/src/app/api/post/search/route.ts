import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _posts } from 'src/_mock/_blog';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Search posts
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const query = searchParams.get('query')?.trim().toLowerCase();

    if (!query) {
      return response({ results: [] }, STATUS.OK);
    }

    const posts = _posts();

    // Accept search by title or description
    const results = posts.filter(
      ({ title, description }) =>
        title.toLowerCase().includes(query) || description?.toLowerCase().includes(query)
    );

    logger('[Post] search-results', results.length);

    return response({ results }, STATUS.OK);
  } catch (error) {
    return handleError('Post - Get search', error);
  }
}
