import type { TFunction } from 'i18next';
import type { UserRechargePackage, PublicPaymentChannel } from 'src/types/recharge';
import type { CaptchaState, RechargeCaptchaConfig } from './wallet-recharge-captcha';

export type WalletRechargeDialogProps = {
  t: TFunction<'admin'>;
  open: boolean;
  loading: boolean;
  enabled: boolean;
  ratio: number;
  minAmount: number;
  maxAmount: number;
  packages: UserRechargePackage[];
  channels: PublicPaymentChannel[];
  channelsLoading: boolean;
  captchaConfig: RechargeCaptchaConfig;
  purchasingId: string | null;
  pendingPaymentOrderNo: string | null;
  checkingPayment: boolean;
  onClose: VoidFunction;
  onPurchaseAmount: (
    amount: number,
    channelCode: string,
    methodCode: string,
    captchaToken?: string
  ) => void;
  onPurchasePackage: (
    item: UserRechargePackage,
    channelCode: string,
    methodCode: string,
    captchaToken?: string
  ) => void;
  onRefresh: VoidFunction;
  onCheckPayment: VoidFunction;
};

export type SelectorState = {
  methodCode: string;
  setMethodCode: (value: string) => void;
};

export type WalletRechargeFormProps = WalletRechargeDialogProps & {
  amount: string;
  selector: SelectorState;
  captcha: CaptchaState;
  onAmountChange: (value: string) => void;
};
