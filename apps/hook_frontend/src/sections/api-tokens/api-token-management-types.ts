import type { SystemUser } from 'src/types/rbac';
import type { GlobalModelResponse } from 'src/types/model';
import type { ApiToken, ApiTokenType, ModelAccessMode } from 'src/types/api-token';

export type TokenScope = 'user' | 'admin';

export type TokenForm = {
  name: string;
  token_type: ApiTokenType;
  user_id: string;
  group_code: string;
  expires_at: string;
  model_access_mode: ModelAccessMode;
  allowed_model_ids: string[];
  rate_limit_rpm: string;
  quota_limit: string;
};

export type TokenFormErrors = Partial<Record<keyof TokenForm, string>>;

export type TokenDialogState = {
  clearError: (field: keyof TokenForm) => void;
  closeCreatedToken: () => void;
  closeDialog: () => void;
  createdToken: string | null;
  creating: boolean;
  editing: ApiToken | null;
  errors: TokenFormErrors;
  form: TokenForm;
  open: boolean;
  openCreate: (defaultGroup: string) => void;
  openEdit: (token: ApiToken) => void;
  setForm: React.Dispatch<React.SetStateAction<TokenForm>>;
  submit: () => Promise<void>;
  submitting: boolean;
};

export type BillingGroupOption = {
  code: string;
  name: string;
  allowed_model_ids: string[];
  visible_user_group_codes: string[];
  is_system: boolean;
};

export type TokenModelOption = Pick<GlobalModelResponse, 'id' | 'name' | 'display_name'>;

export type UserOption = Pick<SystemUser, 'id' | 'username' | 'email' | 'group_code' | 'system'>;
