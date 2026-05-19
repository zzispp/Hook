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

  const statusCodes = new Set<number>();

  return {
    window_seconds: windowSecondsValue,
    rules: rules.map((rule) => rulePayload({ rule, statusCodes, t })),
  };
}

function rulePayload({
  rule,
  statusCodes,
  t,
}: {
  rule: RuleForm;
  statusCodes: Set<number>;
  t: (key: string) => string;
}): ProviderCooldownRule {
  if (ruleIncomplete(rule)) throw new Error(t('messages.providerCooldownRuleRequired'));

  const statusCode = integerValue(rule.status_code);
  const failureCount = positiveInteger(rule.failure_count);
  const cooldownSeconds = positiveInteger(rule.cooldown_seconds);

  if (statusCode === null || statusCode < MIN_STATUS_CODE || statusCode > MAX_STATUS_CODE) {
    throw new Error(t('messages.providerCooldownStatusCodeInvalid'));
  }
  if (failureCount === null || cooldownSeconds === null) {
    throw new Error(t('messages.providerCooldownPositiveValuesRequired'));
  }
  if (statusCodes.has(statusCode))
    throw new Error(t('messages.providerCooldownDuplicateStatusCode'));

  statusCodes.add(statusCode);
  return {
    status_code: statusCode,
    failure_count: failureCount,
    cooldown_seconds: cooldownSeconds,
  };
}

function ruleForm(rule: ProviderCooldownRule): RuleForm {
  return {
    status_code: String(rule.status_code),
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
