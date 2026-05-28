import type { TFunction } from 'i18next';

const UNPAID_LIMIT_ERROR = 'unpaid recharge order limit reached';

export function rechargeErrorMessage(error: unknown, t: TFunction<'admin'>) {
  if (!(error instanceof Error)) {
    return t('messages.saveFailed');
  }
  if (error.message.includes(UNPAID_LIMIT_ERROR)) {
    return t('wallet.recharge.unpaidOrderLimitReached');
  }
  return error.message;
}
