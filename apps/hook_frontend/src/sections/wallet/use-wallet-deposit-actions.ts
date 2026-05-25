'use client';

import type { TFunction } from 'i18next';

import { useState } from 'react';

import { redeemCardCode } from 'src/actions/card-code';
import { createUserRechargeOrder } from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';

type Options = {
  t: TFunction<'admin'>;
  refreshWallet: VoidFunction;
  refreshOrders: () => Promise<unknown> | unknown;
};

export function useWalletDepositActions({ t, refreshWallet, refreshOrders }: Options) {
  const [redeemCode, setRedeemCode] = useState('');
  const [redeeming, setRedeeming] = useState(false);
  const [purchasingId, setPurchasingId] = useState<string | null>(null);

  const submitRedeemCode = async () => {
    if (!redeemCode.trim()) return;
    setRedeeming(true);
    try {
      await redeemCardCode({ code: redeemCode.trim() });
      toast.success(t('wallet.cardCode.redeemed'));
      setRedeemCode('');
      refreshWallet();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setRedeeming(false);
    }
  };

  const purchasePackage = async (packageId: string) => {
    setPurchasingId(packageId);
    try {
      await createUserRechargeOrder({ package_id: packageId });
      toast.success(t('wallet.recharge.orderCreated'));
      await refreshOrders();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setPurchasingId(null);
    }
  };

  return {
    redeemCode,
    redeeming,
    purchasingId,
    setRedeemCode,
    submitRedeemCode,
    purchasePackage,
  };
}
