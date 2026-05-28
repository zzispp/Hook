'use client';

import type { TFunction } from 'i18next';

import { useRef, useState, useEffect } from 'react';

import { redeemCardCode } from 'src/actions/card-code';
import { createUserRechargeOrder } from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';

import { rechargeErrorMessage } from './wallet-recharge-errors';
import { submitPayment, openPaymentWindow } from './wallet-payment-window';
import {
  stopOrderPolling,
  startOrderPolling,
  refreshAndMatchOrder,
  createPaymentPollingState,
} from './wallet-payment-polling';

type Options = {
  t: TFunction<'admin'>;
  rechargeOpen: boolean;
  refreshWallet: VoidFunction;
  refreshOrders: () => Promise<unknown> | unknown;
};

export function useWalletDepositActions({ t, rechargeOpen, refreshWallet, refreshOrders }: Options) {
  const [redeemCode, setRedeemCode] = useState('');
  const [redeeming, setRedeeming] = useState(false);
  const [purchasingId, setPurchasingId] = useState<string | null>(null);
  const [pendingPaymentOrderNo, setPendingPaymentOrderNo] = useState<string | null>(null);
  const [checkingPayment, setCheckingPayment] = useState(false);
  const polling = useRef(createPaymentPollingState(rechargeOpen));

  useEffect(() => {
    polling.current.modalOpen = rechargeOpen;
    if (!rechargeOpen) {
      stopOrderPolling(polling.current);
      setPendingPaymentOrderNo(null);
    }
  }, [rechargeOpen]);

  useEffect(() => () => stopOrderPolling(polling.current), []);

  const submitRedeemCode = async () => {
    if (!redeemCode.trim()) return false;
    setRedeeming(true);
    try {
      await redeemCardCode({ code: redeemCode.trim() });
      toast.success(t('wallet.cardCode.redeemed'));
      setRedeemCode('');
      refreshWallet();
      return true;
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      return false;
    } finally {
      setRedeeming(false);
    }
  };

  const purchasePackage = async (
    packageId: string,
    channelCode: string,
    paymentMethod: string,
    captchaToken?: string
  ) => {
    const paymentWindow = openPaymentWindow(t);
    if (!paymentWindow) {
      return;
    }
    setPurchasingId(packageId);
    try {
      const result = await createUserRechargeOrder({
        package_id: packageId,
        payment_channel_code: channelCode,
        payment_method: paymentMethod,
        ...(captchaToken ? { captcha_token: captchaToken } : {}),
      });
      toast.success(t('wallet.recharge.orderCreated'));
      setPendingPaymentOrderNo(result.order.order_no);
      await refreshOrders();
      submitPayment(result.payment, paymentWindow, t);
      startPolling(result.order.order_no);
    } catch (error) {
      paymentWindow.popup.close();
      toast.error(rechargeErrorMessage(error, t));
    } finally {
      setPurchasingId(null);
    }
  };

  const purchaseAmount = async (
    amount: number,
    channelCode: string,
    paymentMethod: string,
    captchaToken?: string
  ) => {
    const paymentWindow = openPaymentWindow(t);
    if (!paymentWindow) {
      return;
    }
    setPurchasingId('custom');
    try {
      const result = await createUserRechargeOrder({
        recharge_amount: amount,
        payment_channel_code: channelCode,
        payment_method: paymentMethod,
        ...(captchaToken ? { captcha_token: captchaToken } : {}),
      });
      toast.success(t('wallet.recharge.orderCreated'));
      setPendingPaymentOrderNo(result.order.order_no);
      await refreshOrders();
      submitPayment(result.payment, paymentWindow, t);
      startPolling(result.order.order_no);
    } catch (error) {
      paymentWindow.popup.close();
      toast.error(rechargeErrorMessage(error, t));
    } finally {
      setPurchasingId(null);
    }
  };

  const checkPendingPayment = async () => {
    if (!pendingPaymentOrderNo || checkingPayment) {
      return;
    }
    setCheckingPayment(true);
    try {
      const paid = await refreshAndMatchOrder(pendingPaymentOrderNo, refreshOrders, refreshWallet);
      if (paid) {
        toast.success(t('wallet.recharge.paymentReceived'));
        clearPendingPayment();
      } else {
        toast.error(t('wallet.recharge.paymentStillPending'));
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setCheckingPayment(false);
    }
  };

  const clearPendingPayment = () => {
    setPendingPaymentOrderNo(null);
    stopOrderPolling(polling.current);
  };

  const startPolling = (orderNo: string) => {
    startOrderPolling(orderNo, polling.current, refreshOrders, refreshWallet, () => {
      toast.success(t('wallet.recharge.paymentReceived'));
      setPendingPaymentOrderNo(null);
    });
  };

  return {
    redeemCode,
    redeeming,
    purchasingId,
    pendingPaymentOrderNo,
    checkingPayment,
    setRedeemCode,
    submitRedeemCode,
    purchasePackage,
    purchaseAmount,
    checkPendingPayment,
    clearPendingPayment,
  };
}
