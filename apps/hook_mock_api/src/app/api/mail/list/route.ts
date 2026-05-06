import type { NextRequest } from 'next/server';

import { logger } from 'src/utils/logger';
import { STATUS, response, handleError } from 'src/utils/response';

import { _mails, _labels } from 'src/_mock/_mail';

// ----------------------------------------------------------------------

export const runtime = 'edge';

type MailType = ReturnType<typeof _mails>[number];

/** **************************************
 * GET - Mails by labelId
 *************************************** */
export async function GET(req: NextRequest) {
  try {
    const { searchParams } = req.nextUrl;
    const labelId = searchParams.get('labelId');

    const labels = _labels();
    const mails = _mails();

    logger('[Mail] labelId', labelId);

    const label = labels.find((labelItem) => labelItem.id === labelId);

    if (!label) {
      return response({ message: 'Label not found!' }, STATUS.NOT_FOUND);
    }

    // Get filtered mails
    const filteredMails =
      label.type === 'custom'
        ? mails.filter((mail) => mail.labelIds.includes(labelId!))
        : filterMailsByLabelId(mails, labelId);

    logger(`[Mail] label-[${labelId}]`, filteredMails.length);

    return response({ mails: filteredMails }, STATUS.OK);
  } catch (error) {
    return handleError('Mail - Get list', error);
  }
}

/** **************************************
 * Actions & Utility
 *************************************** */
function filterMailsByLabelId(mails: MailType[], labelId?: string | null) {
  if (!labelId || labelId === 'inbox') return mails.filter((mail) => mail.folder === 'inbox');
  if (labelId === 'all') return mails;
  if (labelId === 'starred') return mails.filter((mail) => mail.isStarred);
  if (labelId === 'important') return mails.filter((mail) => mail.isImportant);

  return mails.filter((mail) => mail.folder === labelId);
}
