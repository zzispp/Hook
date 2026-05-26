import type { BillingGroup } from 'src/types/group';

export type GroupForm = {
  code: string;
  name: string;
  description: string;
  billing_multiplier: string;
  allowed_model_ids: string[];
  allowed_provider_ids: string[];
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
  allowed_provider_ids: [],
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
    allowed_provider_ids: group.allowed_provider_ids,
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
    allowed_provider_ids: form.allowed_provider_ids,
    visible_user_group_codes: form.visible_user_group_codes,
    is_active: form.is_active,
    sort_order: Number(form.sort_order || 0),
  };
}
