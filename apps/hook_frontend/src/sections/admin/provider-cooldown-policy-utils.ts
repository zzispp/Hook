import type { ProviderCooldownRule, ProviderCooldownPolicy } from 'src/types/provider';

export type CooldownDurationMode = 'fixed' | 'per_rule';

export type RuleForm = {
  status_code: string;
  failure_count: string;
  cooldown_seconds: string;
};

export type PolicyForm = {
  durationMode: CooldownDurationMode;
  fixedCooldownSeconds: string;
  rules: RuleForm[];
};

type PolicyPayloadOptions = PolicyForm & {
  t: (key: string) => string;
};

const EMPTY_POLICY: ProviderCooldownPolicy = { window_seconds: 0, rules: [] };
const FAILURE_WINDOW_SECONDS = 60;
const MIN_STATUS_CODE = 100;
const MAX_STATUS_CODE = 599;

export function initialPolicyForm(policy?: ProviderCooldownPolicy): PolicyForm {
  const current = policy ?? EMPTY_POLICY;
  const rules = current.rules.map(ruleForm);
  const durationMode = inferDurationMode(current.rules);
  const fixedCooldownSeconds = durationMode === 'fixed' ? (rules[0]?.cooldown_seconds ?? '') : '';

  return {
    durationMode,
    fixedCooldownSeconds,
    rules,
  };
}

export function emptyRule(
  durationMode: CooldownDurationMode,
  fixedCooldownSeconds: string
): RuleForm {
  return {
    status_code: '',
    failure_count: '',
    cooldown_seconds: durationMode === 'fixed' ? fixedCooldownSeconds : '',
  };
}

export function policyPayload({
  durationMode,
  fixedCooldownSeconds,
  rules,
  t,
}: PolicyPayloadOptions): ProviderCooldownPolicy {
  if (rules.length === 0) return EMPTY_POLICY;

  const fixedCooldown = fixedCooldownValue(durationMode, fixedCooldownSeconds, t);
  const statusCodes = new Set<number>();

  return {
    window_seconds: FAILURE_WINDOW_SECONDS,
    rules: rules.map((rule) => rulePayload({ durationMode, fixedCooldown, rule, statusCodes, t })),
  };
}

export function fixedCooldownSeed(rules: RuleForm[]) {
  const first = rules[0]?.cooldown_seconds.trim();
  if (!first) return '';
  return rules.every((rule) => rule.cooldown_seconds.trim() === first) ? first : '';
}

function inferDurationMode(rules: ProviderCooldownRule[]): CooldownDurationMode {
  if (rules.length === 0) return 'fixed';
  const first = rules[0].cooldown_seconds;
  return rules.every((rule) => rule.cooldown_seconds === first) ? 'fixed' : 'per_rule';
}

function rulePayload({
  durationMode,
  fixedCooldown,
  rule,
  statusCodes,
  t,
}: {
  durationMode: CooldownDurationMode;
  fixedCooldown: number | null;
  rule: RuleForm;
  statusCodes: Set<number>;
  t: (key: string) => string;
}): ProviderCooldownRule {
  if (ruleIncomplete(rule, durationMode))
    throw new Error(t('messages.providerCooldownRuleRequired'));

  const statusCode = integerValue(rule.status_code);
  const failureCount = positiveInteger(rule.failure_count);
  const cooldownSeconds = cooldownValue(durationMode, fixedCooldown, rule.cooldown_seconds);

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

function fixedCooldownValue(
  durationMode: CooldownDurationMode,
  fixedCooldownSeconds: string,
  t: (key: string) => string
) {
  if (durationMode === 'per_rule') return null;

  const fixedCooldown = positiveInteger(fixedCooldownSeconds);
  if (fixedCooldown === null) throw new Error(t('messages.providerCooldownFixedDurationRequired'));
  return fixedCooldown;
}

function cooldownValue(
  durationMode: CooldownDurationMode,
  fixedCooldown: number | null,
  cooldownSeconds: string
) {
  return durationMode === 'fixed' ? fixedCooldown : positiveInteger(cooldownSeconds);
}

function ruleForm(rule: ProviderCooldownRule): RuleForm {
  return {
    status_code: String(rule.status_code),
    failure_count: String(rule.failure_count),
    cooldown_seconds: String(rule.cooldown_seconds),
  };
}

function ruleIncomplete(rule: RuleForm, durationMode: CooldownDurationMode) {
  const requiredFieldsMissing = !rule.status_code.trim() || !rule.failure_count.trim();
  return durationMode === 'fixed'
    ? requiredFieldsMissing
    : requiredFieldsMissing || !rule.cooldown_seconds.trim();
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
