import type { ProviderCooldownRule, ProviderCooldownPolicy } from 'src/types/provider';

export type RuleForm = {
  status_code: string;
  failure_count: string;
  cooldown_seconds: string;
};

export type PolicyForm = {
  windowSeconds: string;
  rules: RuleForm[];
};

type PolicyPayloadOptions = PolicyForm & {
  t: (key: string) => string;
};

const EMPTY_POLICY: ProviderCooldownPolicy = { window_seconds: 0, rules: [] };
const DEFAULT_SECONDS = '300';
const MIN_STATUS_CODE = 100;
const MAX_STATUS_CODE = 599;
const STATUS_CODE_RANGE_SEPARATOR = '-';

type StatusCodeRange = {
  start: number;
  end: number;
};

export function initialPolicyForm(policy?: ProviderCooldownPolicy): PolicyForm {
  const current = policy ?? EMPTY_POLICY;

  return {
    windowSeconds: current.window_seconds > 0 ? String(current.window_seconds) : DEFAULT_SECONDS,
    rules: current.rules.map(ruleForm),
  };
}

export function emptyRule(): RuleForm {
  return {
    status_code: '',
    failure_count: '',
    cooldown_seconds: DEFAULT_SECONDS,
  };
}

export function policyPayload({ windowSeconds, rules, t }: PolicyPayloadOptions): ProviderCooldownPolicy {
  if (rules.length === 0) return EMPTY_POLICY;

  const windowSecondsValue = positiveInteger(windowSeconds);
  if (windowSecondsValue === null) throw new Error(t('messages.providerCooldownWindowRequired'));

  const statusCodeRanges: StatusCodeRange[] = [];

  return {
    window_seconds: windowSecondsValue,
    rules: rules.map((rule) => rulePayload({ rule, statusCodeRanges, t })),
  };
}

function rulePayload({
  rule,
  statusCodeRanges,
  t,
}: {
  rule: RuleForm;
  statusCodeRanges: StatusCodeRange[];
  t: (key: string) => string;
}): ProviderCooldownRule {
  if (ruleIncomplete(rule)) throw new Error(t('messages.providerCooldownRuleRequired'));

  const statusCodeRange = parseStatusCodeRange(rule.status_code);
  const failureCount = positiveInteger(rule.failure_count);
  const cooldownSeconds = positiveInteger(rule.cooldown_seconds);

  if (statusCodeRange === null || !validStatusCodeRange(statusCodeRange)) {
    throw new Error(t('messages.providerCooldownStatusCodeInvalid'));
  }
  if (failureCount === null || cooldownSeconds === null) {
    throw new Error(t('messages.providerCooldownPositiveValuesRequired'));
  }
  if (statusCodeRanges.some((range) => rangesOverlap(range, statusCodeRange)))
    throw new Error(t('messages.providerCooldownDuplicateStatusCode'));

  statusCodeRanges.push(statusCodeRange);
  return {
    status_code_start: statusCodeRange.start,
    status_code_end: statusCodeRange.end,
    failure_count: failureCount,
    cooldown_seconds: cooldownSeconds,
  };
}

function ruleForm(rule: ProviderCooldownRule): RuleForm {
  return {
    status_code: formatStatusCodeRange(rule),
    failure_count: String(rule.failure_count),
    cooldown_seconds: String(rule.cooldown_seconds),
  };
}

function ruleIncomplete(rule: RuleForm) {
  return !rule.status_code.trim() || !rule.failure_count.trim() || !rule.cooldown_seconds.trim();
}

function positiveInteger(value: string) {
  const number = integerValue(value);
  return number !== null && number > 0 ? number : null;
}

function integerValue(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const number = Number(trimmed);
  return Number.isInteger(number) ? number : null;
}

function parseStatusCodeRange(value: string): StatusCodeRange | null {
  const parts = value.split(STATUS_CODE_RANGE_SEPARATOR).map((part) => integerValue(part));
  if (parts.length === 1 && parts[0] !== null) return { start: parts[0], end: parts[0] };
  if (parts.length === 2 && parts[0] !== null && parts[1] !== null) {
    return { start: parts[0], end: parts[1] };
  }
  return null;
}

function validStatusCodeRange(range: StatusCodeRange) {
  return (
    range.start >= MIN_STATUS_CODE &&
    range.start <= MAX_STATUS_CODE &&
    range.end >= MIN_STATUS_CODE &&
    range.end <= MAX_STATUS_CODE &&
    range.start <= range.end
  );
}

function rangesOverlap(left: StatusCodeRange, right: StatusCodeRange) {
  return left.start <= right.end && right.start <= left.end;
}

function formatStatusCodeRange(rule: ProviderCooldownRule) {
  if (rule.status_code_start === rule.status_code_end) return String(rule.status_code_start);
  return `${rule.status_code_start}-${rule.status_code_end}`;
}
