'use client';

import type { TFunction } from 'i18next';
import type { UserRechargePackage } from 'src/types/recharge';

import Card from '@mui/material/Card';
import Grid from '@mui/material/Grid';

import { WalletRechargePanel } from './wallet-recharge-panel';
import { WalletCardCodePanel } from './wallet-card-code-panel';

type Props = {
  t: TFunction<'admin'>;
  locale: string;
  rechargeLoading: boolean;
  rechargeEnabled: boolean;
  ratio: number;
  packages: UserRechargePackage[];
  purchasingId: string | null;
  cardCode: string;
  redeeming: boolean;
  onPurchase: (item: UserRechargePackage) => void;
  onRefreshRecharge: VoidFunction;
  onCardCodeChange: (value: string) => void;
  onRedeemCardCode: VoidFunction;
};

export function WalletDepositSection(props: Props) {
  return (
    <Grid container spacing={3} sx={{ mb: 3 }}>
      <Grid size={{ xs: 12, lg: 8 }}>
        <Card sx={{ height: 1 }}>
          <WalletRechargePanel
            t={props.t}
            loading={props.rechargeLoading}
            enabled={props.rechargeEnabled}
            ratio={props.ratio}
            packages={props.packages}
            purchasingId={props.purchasingId}
            onPurchase={props.onPurchase}
            onRefresh={props.onRefreshRecharge}
          />
        </Card>
      </Grid>
      <Grid size={{ xs: 12, lg: 4 }}>
        <WalletCardCodePanel
          t={props.t}
          code={props.cardCode}
          redeeming={props.redeeming}
          onCodeChange={props.onCardCodeChange}
          onRedeem={props.onRedeemCardCode}
        />
      </Grid>
    </Grid>
  );
}
