import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _posts } from 'src/_mock/_blog';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Posts
 *************************************** */
export async function GET() {
  try {
    const posts = _posts();

    logger('[Post] list', posts.length);

    return response({ posts }, STATUS.OK);
  } catch (error) {
    return handleError('Post - Get list', error);
  }
}
