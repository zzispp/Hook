import type { BodyRuleCondition, BodyRuleConditionOp } from 'src/types/provider';

export type ConditionSource = 'current' | 'original';
export type ConditionGroupMode = 'all' | 'any';

export type EditableConditionLeaf = {
  kind: 'leaf';
  path: string;
  op: BodyRuleConditionOp;
  value: string;
  source: ConditionSource;
};

export type EditableConditionGroup = {
  kind: 'group';
  mode: ConditionGroupMode;
  children: EditableConditionNode[];
};

export type EditableConditionNode = EditableConditionLeaf | EditableConditionGroup;

export const CONDITION_OP_OPTIONS: Array<{ value: BodyRuleConditionOp; label: string }> = [
  { value: 'eq', label: '等于' },
  { value: 'neq', label: '不等于' },
  { value: 'gt', label: '大于' },
  { value: 'lt', label: '小于' },
  { value: 'gte', label: '大于等于' },
  { value: 'lte', label: '小于等于' },
  { value: 'starts_with', label: '开头匹配' },
  { value: 'ends_with', label: '结尾匹配' },
  { value: 'contains', label: '包含' },
  { value: 'matches', label: '正则匹配' },
  { value: 'exists', label: '存在' },
  { value: 'not_exists', label: '不存在' },
  { value: 'in', label: '在列表中' },
  { value: 'type_is', label: '类型是' },
];

const NUMERIC_OPS = new Set<BodyRuleConditionOp>(['gt', 'lt', 'gte', 'lte']);
const STRING_OPS = new Set<BodyRuleConditionOp>(['starts_with', 'ends_with']);
const TYPE_IS_VALUES = new Set(['string', 'number', 'boolean', 'array', 'object', 'null']);

export function createEmptyConditionLeaf(): EditableConditionLeaf {
  return { kind: 'leaf', path: '', op: 'eq', value: '', source: 'current' };
}

export function createConditionGroup(
  mode: ConditionGroupMode = 'all',
  children: EditableConditionNode[] = [createEmptyConditionLeaf()]
): EditableConditionGroup {
  return { kind: 'group', mode, children };
}

export function cloneEditableCondition(node: EditableConditionNode): EditableConditionNode {
  if (node.kind === 'leaf') return { ...node };
  return { kind: 'group', mode: node.mode, children: node.children.map(cloneEditableCondition) };
}

export function conditionToEditable(condition?: BodyRuleCondition | null): EditableConditionNode | null {
  if (!condition) return null;
  if ('all' in condition) return createConditionGroup('all', condition.all.map(conditionToEditableNode));
  if ('any' in condition) return createConditionGroup('any', condition.any.map(conditionToEditableNode));
  return {
    kind: 'leaf',
    path: condition.path || '',
    op: condition.op || 'eq',
    value: condition.value === undefined ? '' : stringifyConditionValue(condition.value),
    source: condition.source === 'original' ? 'original' : 'current',
  };
}

export function editableConditionToApi(node: EditableConditionNode | null): BodyRuleCondition | undefined {
  if (!node) return undefined;
  if (node.kind === 'group') return editableGroupToApi(node);
  const path = node.path.trim();
  if (!path) return undefined;
  const base = { path, op: node.op, ...(node.source === 'original' ? { source: 'original' as const } : {}) };
  if (!isConditionValueRequired(node.op)) return base;
  return { ...base, value: parseConditionValue(node.value) };
}

export function conditionEquals(left: EditableConditionNode | null, right: EditableConditionNode | null): boolean {
  if (left === right) return true;
  if (!left || !right || left.kind !== right.kind) return false;
  if (left.kind === 'leaf' && right.kind === 'leaf') return leafEquals(left, right);
  if (left.kind === 'group' && right.kind === 'group') return groupEquals(left, right);
  return false;
}

export function validateEditableCondition(node: EditableConditionNode | null): string | null {
  if (!node) return null;
  if (node.kind === 'group') return validateGroup(node);
  return validateLeaf(node);
}

export function isConditionValueRequired(op: BodyRuleConditionOp) {
  return op !== 'exists' && op !== 'not_exists';
}

export function getConditionValuePlaceholder(op: BodyRuleConditionOp) {
  if (op === 'in') return '["a", "b"]';
  if (op === 'type_is') return 'string/number/boolean/...';
  return '值';
}

function conditionToEditableNode(condition: BodyRuleCondition) {
  return conditionToEditable(condition) || createEmptyConditionLeaf();
}

function stringifyConditionValue(value: unknown) {
  return typeof value === 'string' ? value : JSON.stringify(value);
}

function editableGroupToApi(node: EditableConditionGroup): BodyRuleCondition | undefined {
  const children = node.children
    .map((child) => editableConditionToApi(child))
    .filter((child): child is BodyRuleCondition => Boolean(child));
  if (!children.length) return undefined;
  return node.mode === 'all' ? { all: children } : { any: children };
}

function parseConditionValue(value: string) {
  const raw = value.trim();
  if (!raw) return '';
  try {
    return JSON.parse(raw);
  } catch {
    return raw;
  }
}

function leafEquals(left: EditableConditionLeaf, right: EditableConditionLeaf) {
  return left.path === right.path && left.op === right.op && left.value === right.value && left.source === right.source;
}

function groupEquals(left: EditableConditionGroup, right: EditableConditionGroup) {
  return left.mode === right.mode && left.children.length === right.children.length && left.children.every((child, i) => conditionEquals(child, right.children[i]));
}

function validateGroup(node: EditableConditionGroup) {
  if (!node.children.length) return '组合条件至少需要一个子条件';
  for (let i = 0; i < node.children.length; i += 1) {
    const error = validateEditableCondition(node.children[i]);
    if (error) return `子条件 ${i + 1}: ${error}`;
  }
  return null;
}

function validateLeaf(node: EditableConditionLeaf) {
  if (!node.path.trim()) return '条件路径不能为空';
  if (!isConditionValueRequired(node.op)) return null;
  const parsed = parseConditionValue(node.value);
  if (NUMERIC_OPS.has(node.op) && typeof parsed !== 'number') return '数值条件必须填写数字';
  if (STRING_OPS.has(node.op) && typeof parsed !== 'string') return '该条件值必须为字符串';
  if (node.op === 'matches') return validateRegexCondition(parsed);
  if (node.op === 'in' && !Array.isArray(parsed)) return 'in 条件值必须是 JSON 数组';
  if (node.op === 'type_is' && (typeof parsed !== 'string' || !TYPE_IS_VALUES.has(parsed))) return 'type_is 仅支持 string/number/boolean/array/object/null';
  return null;
}

function validateRegexCondition(value: unknown) {
  if (typeof value !== 'string' || !value) return '正则条件值不能为空';
  try {
    new RegExp(value);
    return null;
  } catch (error) {
    return `正则表达式无效：${error instanceof Error ? error.message : String(error)}`;
  }
}
