'use client';

import type { IconifyName } from 'src/components/iconify';
import type { SystemSettingsForm } from './system-settings-utils';
import type { ContactMethod, ContactMethodType } from 'src/types/system-setting';

export type ContactMethodSetForm = React.Dispatch<React.SetStateAction<SystemSettingsForm>>;

export const CONTACT_TYPE_OPTIONS: ReadonlyArray<ContactMethodType> = [
  'wechat',
  'telegram',
  'discord',
  'qq',
  'qq_group',
  'custom',
];

export const CONTACT_ICON_BY_TYPE: Record<ContactMethodType, IconifyName> = {
  wechat: 'simple-icons:wechat',
  telegram: 'simple-icons:telegram',
  discord: 'simple-icons:discord',
  qq: 'simple-icons:qq',
  qq_group: 'solar:users-group-rounded-bold',
  custom: 'solar:chat-round-dots-bold',
};

export function addContactMethod(setForm: ContactMethodSetForm) {
  setForm((current) => ({
    ...current,
    contact_methods: [...current.contact_methods, defaultContactMethod()],
  }));
}

export function updateContactMethod(
  setForm: ContactMethodSetForm,
  index: number,
  method: ContactMethod
) {
  setForm((current) => ({
    ...current,
    contact_methods: current.contact_methods.map((item, itemIndex) =>
      itemIndex === index ? method : item
    ),
  }));
}

export function removeContactMethod(setForm: ContactMethodSetForm, index: number) {
  setForm((current) => ({
    ...current,
    contact_methods: current.contact_methods.filter((_item, itemIndex) => itemIndex !== index),
  }));
}

export function moveContactMethod(
  setForm: ContactMethodSetForm,
  index: number,
  direction: -1 | 1
) {
  setForm((current) => {
    const next = [...current.contact_methods];
    const target = index + direction;
    [next[index], next[target]] = [next[target], next[index]];
    return { ...current, contact_methods: next };
  });
}

export function handleQrCodeUpload(
  event: React.ChangeEvent<HTMLInputElement>,
  method: ContactMethod,
  onChange: (method: ContactMethod) => void
) {
  const file = event.target.files?.[0];
  event.target.value = '';
  if (!file) {
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    if (typeof reader.result === 'string') {
      onChange({ ...method, qr_code: reader.result });
    }
  };
  reader.readAsDataURL(file);
}

function defaultContactMethod(): ContactMethod {
  return {
    id: crypto.randomUUID(),
    type: 'wechat',
    custom_type: '',
    icon: CONTACT_ICON_BY_TYPE.wechat,
    value: '',
    qr_code: '',
  };
}
