import type { BodyRule } from 'src/types/provider';
import type { EditableBodyRule } from './provider-endpoint-rule-types';

import {
  bodyRulesToEditable,
  editableBodyRulesToApi,
} from './provider-endpoint-rule-types';

export const OPENAI_COMPACT_API_FORMAT = 'openai_compact';

const OPENAI_COMPACT_DEFAULT_BODY_RULES: BodyRule[] = [
  {
    action: 'drop',
    path: 'max_output_tokens',
    condition: { path: 'max_output_tokens', op: 'exists', source: 'current' },
  },
  {
    action: 'drop',
    path: 'temperature',
    condition: { path: 'temperature', op: 'exists', source: 'current' },
  },
  {
    action: 'drop',
    path: 'top_p',
    condition: { path: 'top_p', op: 'exists', source: 'current' },
  },
  { action: 'set', path: 'store', value: false },
  {
    action: 'set',
    path: 'instructions',
    value: 'You are GPT-5.',
    condition: { path: 'instructions', op: 'not_exists', source: 'current' },
  },
];

export function defaultOpenAiCompactBodyRules(): EditableBodyRule[] {
  return bodyRulesToEditable(OPENAI_COMPACT_DEFAULT_BODY_RULES);
}

export function isDefaultOpenAiCompactBodyRules(
  rules: EditableBodyRule[]
): boolean {
  return (
    JSON.stringify(editableBodyRulesToApi(rules)) ===
    JSON.stringify(OPENAI_COMPACT_DEFAULT_BODY_RULES)
  );
}
