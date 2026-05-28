'use client';

import type { TFunction } from 'i18next';
import type { PaymentOrderAction } from 'src/types/recharge';

import { toast } from 'src/components/snackbar';

export type PaymentWindowHandle = {
  popup: Window;
};

export function openPaymentWindow(t: TFunction<'admin'>): PaymentWindowHandle | null {
  const popup = window.open('about:blank', '_blank');
  if (!popup) {
    toast.error(t('wallet.recharge.paymentWindowBlocked'));
    return null;
  }
  writePaymentWindowShell(popup, t('wallet.recharge.paymentWindowCreating'));
  return { popup };
}

export function submitPayment(
  payment: PaymentOrderAction,
  paymentWindow: PaymentWindowHandle,
  t: TFunction<'admin'>
) {
  paymentWindow.popup.document.open();
  paymentWindow.popup.document.write(
    paymentFormHtml(payment, t('wallet.recharge.paymentWindowRedirecting'))
  );
  paymentWindow.popup.document.close();
}

function writePaymentWindowShell(paymentWindow: Window, message: string) {
  paymentWindow.document.open();
  paymentWindow.document.write(paymentWindowShellHtml(message));
  paymentWindow.document.close();
}

function paymentWindowShellHtml(message: string) {
  return `<!doctype html><html><head><meta charset="utf-8"><title>${escapeHtml(message)}</title></head><body><p>${escapeHtml(message)}</p></body></html>`;
}

function paymentFormHtml(payment: PaymentOrderAction, message: string) {
  const fields = Object.entries(payment.fields)
    .map(
      ([name, value]) =>
        `<input type="hidden" name="${escapeHtml(name)}" value="${escapeHtml(value)}">`
    )
    .join('');
  return `<!doctype html><html><head><meta charset="utf-8"><title>${escapeHtml(message)}</title></head><body><p>${escapeHtml(message)}</p><form id="payment-form" method="${escapeHtml(payment.method)}" action="${escapeHtml(payment.action)}">${fields}</form><script>document.getElementById('payment-form').submit();</script></body></html>`;
}

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('"', '&quot;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}
