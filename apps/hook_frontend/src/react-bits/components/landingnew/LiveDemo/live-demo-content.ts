export type ModelKey = 'gpt-4o' | 'claude-3-5-sonnet' | 'gemini-1-5-pro' | 'deepseek-r1';
export type PresetPromptKey = 'gateway' | 'joke' | 'rust';

type TranslateFn = (key: string, options?: Record<string, unknown>) => string;

export type PresetPrompt = {
  readonly key: PresetPromptKey;
  readonly label: string;
  readonly text: string;
};

const PRESET_KEYS: readonly PresetPromptKey[] = ['gateway', 'joke', 'rust'];

const MODEL_OPTION_KEYS: Readonly<Record<ModelKey, string>> = {
  'gpt-4o': 'gpt4o',
  'claude-3-5-sonnet': 'claude',
  'gemini-1-5-pro': 'gemini',
  'deepseek-r1': 'deepseek',
};

export function getPresetPrompts(t: TranslateFn): readonly PresetPrompt[] {
  return PRESET_KEYS.map((key) => ({
    key,
    label: t(`liveDemo.presets.${key}.label`),
    text: t(`liveDemo.presets.${key}.text`),
  }));
}

export function getModelOptionLabel(t: TranslateFn, model: ModelKey): string {
  return t(`liveDemo.modelOptions.${MODEL_OPTION_KEYS[model]}`);
}

export function findPresetByPrompt(
  presetPrompts: readonly PresetPrompt[],
  promptText: string
): PresetPrompt | undefined {
  return presetPrompts.find((item) => item.text === promptText);
}

export function getResponseBody(
  t: TranslateFn,
  model: ModelKey,
  promptText: string,
  presetPrompt?: PresetPrompt
): string {
  const modelKey = MODEL_OPTION_KEYS[model];

  if (presetPrompt) {
    return t(`liveDemo.responses.${modelKey}.${presetPrompt.key}`);
  }

  if (model === 'deepseek-r1') {
    return t('liveDemo.responses.custom.deepseek', { prompt: promptText, model });
  }

  return t('liveDemo.responses.custom.default', { prompt: promptText, model });
}
