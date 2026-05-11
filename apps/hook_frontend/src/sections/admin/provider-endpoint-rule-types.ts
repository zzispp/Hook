import type { EditableConditionNode } from './provider-endpoint-rule-condition';
import type { BodyRule, HeaderRule, BodyRuleNameStyle } from 'src/types/provider';

import {
  conditionToEditable,
  editableConditionToApi,
  validateEditableCondition,
} from './provider-endpoint-rule-condition';

export type HeaderRuleAction = 'set' | 'drop' | 'rename';
export type EditableBodyRuleAction =
  | 'set'
  | 'drop'
  | 'rename'
  | 'insert'
  | 'regex_replace'
  | 'name_style';

export type EditableHeaderRule = {
  action: HeaderRuleAction;
  key: string;
  value: string;
  from: string;
  to: string;
  condition: EditableConditionNode | null;
};

export type EditableBodyRule = {
  action: EditableBodyRuleAction;
  path: string;
  value: string;
  from: string;
  to: string;
  index: string;
  pattern: string;
  replacement: string;
  flags: string;
  style: string;
  condition: EditableConditionNode | null;
};

export type EndpointEditState = {
  baseUrl: string;
  customPath: string;
  headerRules: EditableHeaderRule[];
  bodyRules: EditableBodyRule[];
};

export const HEADER_ACTION_OPTIONS = [
  { value: 'set', label: '覆写' },
  { value: 'drop', label: '删除' },
  { value: 'rename', label: '重命名' },
] satisfies Array<{ value: HeaderRuleAction; label: string }>;

export const BODY_ACTION_OPTIONS = [
  { value: 'set', label: '覆写' },
  { value: 'drop', label: '删除' },
  { value: 'rename', label: '重命名' },
  { value: 'insert', label: '插入' },
  { value: 'regex_replace', label: '正则替换' },
  { value: 'name_style', label: '命名风格' },
] satisfies Array<{ value: EditableBodyRuleAction; label: string }>;

export const BODY_NAME_STYLE_OPTIONS = [
  { value: 'snake_case', label: 'snake_case' },
  { value: 'camelCase', label: 'camelCase' },
  { value: 'PascalCase', label: 'PascalCase' },
  { value: 'kebab-case', label: 'kebab-case' },
  { value: 'capitalize', label: 'Capitalize' },
] satisfies Array<{ value: BodyRuleNameStyle; label: string }>;

const ORIGINAL_PLACEHOLDER = '{{$original}}';
const ORIGINAL_SENTINEL = '__HOOK_ORIGINAL__';

export function emptyHeaderRule(action: HeaderRuleAction = 'set'): EditableHeaderRule {
  return { action, key: '', value: '', from: '', to: '', condition: null };
}

export function emptyBodyRule(action: EditableBodyRuleAction = 'set'): EditableBodyRule {
  return {
    action,
    path: '',
    value: '',
    from: '',
    to: '',
    index: '',
    pattern: '',
    replacement: '',
    flags: '',
    style: action === 'name_style' ? 'capitalize' : '',
    condition: null,
  };
}

export function headerRulesToEditable(rules?: HeaderRule[] | null): EditableHeaderRule[] {
  return (rules ?? []).map(headerRuleToEditable);
}

export function bodyRulesToEditable(rules?: BodyRule[] | null): EditableBodyRule[] {
  return (rules ?? []).map(bodyRuleToEditable);
}

export function editableHeaderRulesToApi(rules: EditableHeaderRule[]): HeaderRule[] | null {
  const result = rules.map(editableHeaderRuleToApi).filter((rule): rule is HeaderRule => Boolean(rule));
  return result.length ? result : null;
}

export function editableBodyRulesToApi(rules: EditableBodyRule[]): BodyRule[] | null {
  const result = rules.map(editableBodyRuleToApi).filter((rule): rule is BodyRule => Boolean(rule));
  return result.length ? result : null;
}

export function validateHeaderRules(rules: EditableHeaderRule[]) {
  for (let i = 0; i < rules.length; i += 1) {
    const error = validateHeaderRule(rules[i]);
    if (error) return `第 ${i + 1} 条请求头规则：${error}`;
  }
  return null;
}

export function validateBodyRules(rules: EditableBodyRule[]) {
  for (let i = 0; i < rules.length; i += 1) {
    const error = validateBodyRule(rules[i]);
    if (error) return `第 ${i + 1} 条请求体规则：${error}`;
  }
  return null;
}

export function headerRulesChanged(original?: HeaderRule[] | null, rules: EditableHeaderRule[] = []) {
  return JSON.stringify(original ?? null) !== JSON.stringify(editableHeaderRulesToApi(rules));
}

export function bodyRulesChanged(original?: BodyRule[] | null, rules: EditableBodyRule[] = []) {
  return JSON.stringify(original ?? null) !== JSON.stringify(editableBodyRulesToApi(rules));
}

export function getBodyJsonValidation(rule: EditableBodyRule): boolean | null {
  if (!bodyRuleNeedsJsonValue(rule)) return null;
  if (!rule.value.trim()) return null;
  try {
    parseRuleJsonValue(rule.value);
    return true;
  } catch {
    return false;
  }
}

export function getRegexValidation(rule: EditableBodyRule): boolean | null {
  if (rule.action !== 'regex_replace' || !rule.pattern.trim()) return null;
  try {
    new RegExp(rule.pattern.trim());
    return rule.flags.trim() ? /^[ims]+$/.test(rule.flags.trim()) : true;
  } catch {
    return false;
  }
}

function headerRuleToEditable(rule: HeaderRule): EditableHeaderRule {
  const condition = conditionToEditable(rule.condition);
  if (rule.action === 'set') return { ...emptyHeaderRule('set'), key: rule.key, value: rule.value ?? '', condition };
  if (rule.action === 'drop') return { ...emptyHeaderRule('drop'), key: rule.key, condition };
  return { ...emptyHeaderRule('rename'), from: rule.from, to: rule.to, condition };
}

function bodyRuleToEditable(rule: BodyRule): EditableBodyRule {
  const condition = conditionToEditable(rule.condition);
  if (rule.action === 'set') return { ...emptyBodyRule('set'), path: rule.path, value: stringifyRuleValue(rule.value), condition };
  if (rule.action === 'drop') return { ...emptyBodyRule('drop'), path: rule.path, condition };
  if (rule.action === 'rename') return { ...emptyBodyRule('rename'), from: rule.from, to: rule.to, condition };
  if (rule.action === 'append') return { ...emptyBodyRule('insert'), path: rule.path, value: stringifyRuleValue(rule.value), condition };
  if (rule.action === 'insert') return { ...emptyBodyRule('insert'), path: rule.path, index: String(rule.index), value: stringifyRuleValue(rule.value), condition };
  if (rule.action === 'regex_replace') return regexRuleToEditable(rule, condition);
  return { ...emptyBodyRule('name_style'), path: rule.path, style: rule.style, condition };
}

function editableHeaderRuleToApi(rule: EditableHeaderRule): HeaderRule | null {
  const condition = editableConditionToApi(rule.condition);
  if (rule.action === 'set' && rule.key.trim()) return { action: 'set', key: rule.key.trim(), value: rule.value, ...(condition ? { condition } : {}) };
  if (rule.action === 'drop' && rule.key.trim()) return { action: 'drop', key: rule.key.trim(), ...(condition ? { condition } : {}) };
  if (rule.action === 'rename' && rule.from.trim() && rule.to.trim()) return { action: 'rename', from: rule.from.trim(), to: rule.to.trim(), ...(condition ? { condition } : {}) };
  return null;
}

function editableBodyRuleToApi(rule: EditableBodyRule): BodyRule | null {
  const condition = editableConditionToApi(rule.condition);
  const withCondition = condition ? { condition } : {};
  if (rule.action === 'set' && rule.path.trim()) return { action: 'set', path: rule.path.trim(), value: parseRuleJsonValue(rule.value), ...withCondition };
  if (rule.action === 'drop' && rule.path.trim()) return { action: 'drop', path: rule.path.trim(), ...withCondition };
  if (rule.action === 'rename' && rule.from.trim() && rule.to.trim()) return { action: 'rename', from: rule.from.trim(), to: rule.to.trim(), ...withCondition };
  if (rule.action === 'insert' && rule.path.trim()) return editableInsertRuleToApi(rule, withCondition);
  if (rule.action === 'regex_replace' && rule.path.trim() && rule.pattern.trim()) return editableRegexRuleToApi(rule, withCondition);
  if (rule.action === 'name_style' && rule.path.trim() && rule.style.trim()) return { action: 'name_style', path: rule.path.trim(), style: rule.style as BodyRuleNameStyle, ...withCondition };
  return null;
}

function validateHeaderRule(rule: EditableHeaderRule) {
  if (rule.action !== 'rename' && !rule.key.trim()) return '请求头不能为空';
  if (rule.action === 'rename' && (!rule.from.trim() || !rule.to.trim())) return '重命名需要来源和目标请求头';
  return validateEditableCondition(rule.condition);
}

function validateBodyRule(rule: EditableBodyRule) {
  if (rule.action === 'rename' && (!rule.from.trim() || !rule.to.trim())) return '重命名需要来源和目标路径';
  if (rule.action !== 'rename' && !rule.path.trim()) return '字段路径不能为空';
  const actionError = validateBodyRuleAction(rule);
  if (actionError) return actionError;
  return validateEditableCondition(rule.condition);
}

function validateBodyRuleAction(rule: EditableBodyRule) {
  if (bodyRuleNeedsJsonValue(rule)) return validateRuleJsonValue(rule.value);
  if (rule.action === 'regex_replace') return validateRegexRule(rule);
  if (rule.action === 'name_style' && !BODY_NAME_STYLE_OPTIONS.some((item) => item.value === rule.style)) return '请选择有效的命名风格';
  if (rule.action === 'insert' && rule.index.trim() && Number.isNaN(Number(rule.index))) return '位置必须为整数或留空';
  return null;
}

function bodyRuleNeedsJsonValue(rule: EditableBodyRule) {
  return rule.action === 'set' || rule.action === 'insert';
}

function validateRuleJsonValue(value: string) {
  if (!value.trim()) return '值不能为空';
  try {
    parseRuleJsonValue(value);
    return null;
  } catch (error) {
    return `JSON 格式错误：${error instanceof Error ? error.message : String(error)}`;
  }
}

function validateRegexRule(rule: EditableBodyRule) {
  if (!rule.pattern.trim()) return '正则表达式不能为空';
  if (getRegexValidation(rule) === false) return '正则表达式或 flags 无效';
  return null;
}

function regexRuleToEditable(
  rule: Extract<BodyRule, { action: 'regex_replace' }>,
  condition: EditableConditionNode | null
) {
  return {
    ...emptyBodyRule('regex_replace'),
    path: rule.path,
    pattern: rule.pattern,
    replacement: rule.replacement,
    flags: rule.flags ?? '',
    condition,
  };
}

function editableInsertRuleToApi(rule: EditableBodyRule, condition: { condition?: BodyRule['condition'] }) {
  const value = parseRuleJsonValue(rule.value);
  const index = rule.index.trim();
  if (!index) return { action: 'append', path: rule.path.trim(), value, ...condition } satisfies BodyRule;
  return { action: 'insert', path: rule.path.trim(), index: Number(index), value, ...condition } satisfies BodyRule;
}

function editableRegexRuleToApi(rule: EditableBodyRule, condition: { condition?: BodyRule['condition'] }) {
  return {
    action: 'regex_replace',
    path: rule.path.trim(),
    pattern: rule.pattern,
    replacement: rule.replacement,
    ...(rule.flags.trim() ? { flags: rule.flags.trim() } : {}),
    ...condition,
  } satisfies BodyRule;
}

function stringifyRuleValue(value: unknown) {
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function parseRuleJsonValue(value: string) {
  return restoreOriginalPlaceholder(JSON.parse(prepareOriginalPlaceholder(value.trim())));
}

function prepareOriginalPlaceholder(value: string) {
  const replaced = value.replaceAll(ORIGINAL_PLACEHOLDER, ORIGINAL_SENTINEL);
  try {
    JSON.parse(replaced);
    return replaced;
  } catch {
    return replaced.replaceAll(ORIGINAL_SENTINEL, `"${ORIGINAL_SENTINEL}"`);
  }
}

function restoreOriginalPlaceholder(value: unknown): unknown {
  if (typeof value === 'string') return value.replaceAll(ORIGINAL_SENTINEL, ORIGINAL_PLACEHOLDER);
  if (Array.isArray(value)) return value.map(restoreOriginalPlaceholder);
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, restoreOriginalPlaceholder(item)]));
}
