'use client';

import type { SystemSettingsForm } from './system-settings-utils';

import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';
import { usePaymentChannels, updatePaymentChannel } from 'src/actions/recharge';

import { toast } from 'src/components/snackbar';

import { SwitchRow, TextFieldRow } from './shared';
import { SettingsSection } from './system-settings-section';

type Props = {
  form: SystemSettingsForm;
  setForm: React.Dispatch<React.SetStateAction<SystemSettingsForm>>;
};

export function RechargeSettingsSection({ form, setForm }: Props) {
  const { t } = useTranslate('admin');
  const channels = usePaymentChannels();

  return (
    <SettingsSection
      title={t('systemSettings.sections.recharge')}
      description={t('systemSettings.recharge.description')}
    >
      <Stack spacing={2}>
        <SwitchRow
          checked={form.recharge_enabled}
          label={t('systemSettings.fields.rechargeEnabled')}
          onChange={(checked) => setForm((current) => ({ ...current, recharge_enabled: checked }))}
        />
        <RechargeAmountFields form={form} setForm={setForm} />
        <Typography variant="body2" color="text.secondary">
          {t('systemSettings.recharge.preview', { ratio: ratioLabel(form.recharge_arrival_ratio) })}
        </Typography>
        <Divider />
        <PaymentChannelList
          loading={channels.isLoading}
          errorMessage={channels.error?.message}
          channels={channels.data ?? []}
          onChanged={() => void channels.refresh()}
        />
      </Stack>
    </SettingsSection>
  );
}

function RechargeAmountFields({ form, setForm }: Props) {
  const { t } = useTranslate('admin');
  const setField = (field: keyof SystemSettingsForm, value: string) =>
    setForm((current) => ({ ...current, [field]: value }));

  return (
    <>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeArrivalRatio')}
          value={form.recharge_arrival_ratio}
          onChange={(value) => setField('recharge_arrival_ratio', value)}
          slotProps={{ htmlInput: { min: 0, step: 0.01 } }}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeOrderExpireMinutes')}
          value={form.recharge_order_expire_minutes}
          onChange={(value) => setField('recharge_order_expire_minutes', value)}
          slotProps={{ htmlInput: { min: 1, step: 1 } }}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeMinAmount')}
          value={form.recharge_min_amount}
          onChange={(value) => setField('recharge_min_amount', value)}
          slotProps={{ htmlInput: { min: 0.01, step: 0.01 } }}
        />
        <TextFieldRow
          type="number"
          label={t('systemSettings.fields.rechargeMaxAmount')}
          value={form.recharge_max_amount}
          onChange={(value) => setField('recharge_max_amount', value)}
          slotProps={{ htmlInput: { min: 0.01, step: 0.01 } }}
        />
      </Stack>
    </>
  );
}

function PaymentChannelList({
  loading,
  errorMessage,
  channels,
  onChanged,
}: {
  loading: boolean;
  errorMessage?: string;
  channels: NonNullable<ReturnType<typeof usePaymentChannels>['data']>;
  onChanged: VoidFunction;
}) {
  const { t } = useTranslate('admin');

  if (errorMessage) {
    return <Typography color="error">{errorMessage}</Typography>;
  }

  if (loading) {
    return <Typography color="text.secondary">{t('common.loading')}</Typography>;
  }

  if (channels.length === 0) {
    return (
      <Typography variant="body2" color="text.secondary">
        {t('systemSettings.recharge.noPaymentChannels')}
      </Typography>
    );
  }

  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('systemSettings.recharge.paymentChannels')}</Typography>
      {channels.map((channel) => (
        <FormControlLabel
          key={channel.code}
          control={
            <Switch
              checked={channel.enabled}
              onChange={(event) =>
                void updateChannel(channel.code, event.target.checked, t, onChanged)
              }
            />
          }
          label={`${channel.name} (${channel.code})`}
        />
      ))}
    </Stack>
  );
}

async function updateChannel(
  code: string,
  enabled: boolean,
  t: ReturnType<typeof useTranslate>['t'],
  onChanged: VoidFunction
) {
  try {
    await updatePaymentChannel(code, { enabled });
    toast.success(t('adminRecharges.messages.paymentChannelUpdated'));
    onChanged();
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
  }
}

function ratioLabel(value: string) {
  const ratio = Number(value || 0);
  if (!Number.isFinite(ratio)) return value;
  return ratio.toString();
}
