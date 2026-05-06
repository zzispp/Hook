import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _navItems } from 'src/_mock/_navbar';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Nav items
 *************************************** */
export async function GET() {
  try {
    logger('[Nav] items', _navItems.length);

    return response({ navItems: _navItems }, STATUS.OK);
  } catch (error) {
    return handleError('Nav - Get list', error);
  }
}
