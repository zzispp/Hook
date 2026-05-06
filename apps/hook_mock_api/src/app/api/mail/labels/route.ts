import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _labels } from 'src/_mock/_mail';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Labels
 *************************************** */
export async function GET() {
  try {
    const labels = _labels();

    logger('[Mail] labels', labels.length);

    return response({ labels }, STATUS.OK);
  } catch (error) {
    return handleError('Mail - Get labels', error);
  }
}
