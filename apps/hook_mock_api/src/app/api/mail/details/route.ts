import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _mails } from 'src/_mock/_mail';

// ----------------------------------------------------------------------

export const runtime = 'edge';

/** **************************************
 * GET - Mail details
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const mailId = searchParams.get('mailId');

    const mails = _mails();

    const mail = mails.find((mailItem) => mailItem.id === mailId);

    if (!mail) {
      return response({ message: 'Mail not found!' }, STATUS.NOT_FOUND);
    }

    logger('[Mail] details', mail.id);

    return response({ mail }, STATUS.OK);
  } catch (error) {
    return handleError('Mail - Get details', error);
  }
}
