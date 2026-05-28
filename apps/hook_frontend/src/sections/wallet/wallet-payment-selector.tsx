'use client';

import type { TFunction } from 'i18next';
import type { PublicPaymentChannel } from 'src/types/recharge';

import { useEffect } from 'react';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';

import { TextFieldRow } from 'src/sections/admin/shared';

type Props = {
  t: TFunction<'admin'>;
  channels: PublicPaymentChannel[];
  methodCode: string;
  disabled: boolean;
  onMethodChange: (value: string) => void;
};

export function WalletPaymentSelector(props: Props) {
  const methods = visibleMethods(props.channels);
  const { channels, methodCode, onMethodChange } = props;

  useEffect(() => {
    syncSelection({
      channels,
      methodCode,
      onMethodChange,
    });
  }, [channels, methodCode, onMethodChange]);

  return (
    <Stack spacing={2}>
      <TextFieldRow
        select
        disabled={props.disabled || methods.length === 0}
        label={props.t('wallet.recharge.paymentMethod')}
        value={props.methodCode}
        onChange={props.onMethodChange}
      >
        {methods.map((method) => (
          <MenuItem key={method.code} value={method.code}>
            {method.name}
          </MenuItem>
        ))}
      </TextFieldRow>
    </Stack>
  );
}

export function paymentChannelForMethod(channels: PublicPaymentChannel[], methodCode: string) {
  return channels.find((channel) => channel.methods.some((method) => method.code === methodCode))?.code ?? '';
}

function syncSelection(props: Pick<Props, 'channels' | 'methodCode' | 'onMethodChange'>) {
  const methods = visibleMethods(props.channels);
  if (methods.length === 0) {
    clearSelection(props);
    return;
  }
  const methodExists = methods.some((method) => method.code === props.methodCode);
  if (!methodExists) {
    props.onMethodChange(methods[0].code);
  }
}

function clearSelection(props: Pick<Props, 'methodCode' | 'onMethodChange'>) {
  if (props.methodCode) {
    props.onMethodChange('');
  }
}

function visibleMethods(channels: PublicPaymentChannel[]) {
  return channels.flatMap((channel) => channel.methods).filter((method, index, methods) => methods.findIndex((item) => item.code === method.code) === index);
}
