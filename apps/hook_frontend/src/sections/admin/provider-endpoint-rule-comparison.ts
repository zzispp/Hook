import type { BodyRule, HeaderRule } from 'src/types/provider';
import type { EditableBodyRule, EditableHeaderRule } from './provider-endpoint-rule-types';

import { bodyRulesToEditable, editableHeaderRulesToApi } from './provider-endpoint-rule-types';

type NormalizedBodyRule = {
  action: EditableBodyRule['action'];
  path: string;
  value: unknown;
  valueInvalid: boolean;
  from: string;
  to: string;
  index: number | null;
  indexInvalid: boolean;
  pattern: string;
  replacement: string;
  flags: string;
  style: string;
  condition: EditableBodyRule['condition'];
};

const ORIGINAL_PLACEHOLDER = '{{$original}}';
const ORIGINAL_SENTINEL = '__HOOK_ORIGINAL__';

export function headerRulesChanged(original?: HeaderRule[] | null, rules: EditableHeaderRule[] = []) {
  return JSON.stringify(original ?? null) !== JSON.stringify(editableHeaderRulesToApi(rules));
}

export function bodyRulesChanged(original?: BodyRule[] | null, rules: EditableBodyRule[] = []) {
  return JSON.stringify(normalizeBodyRulesForComparison(bodyRulesToEditable(original))) !== JSON.stringify(normalizeBodyRulesForComparison(rules));
}

function normalizeBodyRulesForComparison(rules: EditableBodyRule[]) {
  return rules.map((rule) => ({
    action: rule.action,
    path: rule.path.trim(),
    ...normalizeBodyRuleValue(rule),
    from: rule.from.trim(),
    to: rule.to.trim(),
    ...normalizeBodyRuleIndex(rule),
    pattern: rule.pattern.trim(),
    replacement: rule.replacement,
    flags: rule.flags.trim(),
    style: rule.style,
    condition: rule.condition,
  })) satisfies NormalizedBodyRule[];
}

function normalizeBodyRuleValue(rule: EditableBodyRule) {
  if (rule.action !== 'set' && rule.action !== 'insert') {
    return { value: rule.value.trim(), valueInvalid: false };
  }
  const parsed = tryParseRuleJsonValue(rule.value);
  return parsed.ok ? { value: parsed.value, valueInvalid: false } : { value: null, valueInvalid: true };
}

function normalizeBodyRuleIndex(rule: EditableBodyRule) {
  if (rule.action !== 'insert') {
    return { index: null, indexInvalid: false };
  }
  const index = rule.index.trim();
  if (!index) {
    return { index: null, indexInvalid: false };
  }
  const parsed = Number(index);
  return Number.isNaN(parsed) ? { index: null, indexInvalid: true } : { index: parsed, indexInvalid: false };
}

function tryParseRuleJsonValue(value: string): { ok: true; value: unknown } | { ok: false } {
  const trimmed = value.trim();
  if (!trimmed) return { ok: false };
  const replaced = prepareOriginalPlaceholder(trimmed);
  try {
    return { ok: true, value: restoreOriginalPlaceholder(JSON.parse(replaced)) };
  } catch {
    return { ok: false };
  }
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
