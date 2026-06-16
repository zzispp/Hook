import type { SystemUser } from 'src/types/rbac';
import type { ApiToken } from 'src/types/api-token';
import type { BillingGroup } from 'src/types/group';
import type { GlobalModelResponse } from 'src/types/model';

type RoutingModelAccessInput = {
  token: ApiToken | null;
  group: BillingGroup | null;
  user: SystemUser | null;
  models: GlobalModelResponse[];
};

export function routingModelsForToken({
  token,
  group,
  user,
  models,
}: RoutingModelAccessInput) {
  if (!token || !groupAllowsToken(group, token, user)) return [];

  return models.filter(
    (model) =>
      model.is_active &&
      idsAllow(token.allowed_model_ids, model.id, token.model_access_mode === 'all') &&
      idsAllow(group?.allowed_model_ids ?? [], model.id) &&
      userAllowsModel(token, user, model.id)
  );
}

function groupAllowsToken(group: BillingGroup | null, token: ApiToken, user: SystemUser | null) {
  if (!group?.is_active) return false;
  if (token.token_type !== 'user') return true;
  if (!user?.is_active) return false;
  return group.visible_user_group_codes.some((code) => user.group_codes.includes(code));
}

function userAllowsModel(token: ApiToken, user: SystemUser | null, modelId: string) {
  if (token.token_type !== 'user') return true;
  return idsAllow(user?.allowed_model_ids ?? [], modelId);
}

function idsAllow(ids: string[], id: string, forceAllow = false) {
  return forceAllow || ids.length === 0 || ids.includes(id);
}
