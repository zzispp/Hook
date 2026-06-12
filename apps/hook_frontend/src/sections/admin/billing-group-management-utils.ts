import type { BillingGroup } from 'src/types/group';

export type BillingAccessMode = 'unrestricted' | 'provider_key_groups';

export type GroupForm = {
  code: string;
  name: string;
  description: string;
  billing_multiplier: string;
  allowed_model_ids: string[];
  access_mode: BillingAccessMode;
  allowed_provider_key_group_ids: string[];
  visible_user_group_codes: string[];
  is_active: boolean;
  sort_order: string;
};

export const DEFAULT_GROUP_FORM: GroupForm = {
  code: '',
  name: '',
  description: '',
  billing_multiplier: '1',
  allowed_model_ids: [],
  access_mode: 'unrestricted',
  allowed_provider_key_group_ids: [],
  visible_user_group_codes: ['default'],
  is_active: true,
  sort_order: '0',
};

export function formFromGroup(group: BillingGroup): GroupForm {
  return {
    code: group.code,
    name: group.name,
    description: group.description ?? '',
    billing_multiplier: String(group.billing_multiplier),
    allowed_model_ids: group.allowed_model_ids,
    access_mode: accessModeFromGroup(group),
    allowed_provider_key_group_ids: group.allowed_provider_key_group_ids,
    visible_user_group_codes: group.visible_user_group_codes,
    is_active: group.is_active,
    sort_order: String(group.sort_order),
  };
}

export function groupPayload(form: GroupForm) {
  return {
    name: form.name,
    description: form.description.trim() || null,
    billing_multiplier: Number(form.billing_multiplier),
    allowed_model_ids: form.allowed_model_ids,
    allowed_provider_key_group_ids: providerKeyGroupIdsForPayload(form),
    visible_user_group_codes: form.visible_user_group_codes,
    is_active: form.is_active,
    sort_order: Number(form.sort_order || 0),
  };
}

function accessModeFromGroup(group: BillingGroup): BillingAccessMode {
  if (group.allowed_provider_key_group_ids.length > 0) return 'provider_key_groups';
  return 'unrestricted';
}

function providerKeyGroupIdsForPayload(form: GroupForm) {
  return form.access_mode === 'provider_key_groups' ? form.allowed_provider_key_group_ids : [];
}
