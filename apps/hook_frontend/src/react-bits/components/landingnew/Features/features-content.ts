type TranslateFn = (key: string) => string;

export type MonitorToken = {
  readonly cls: string;
  readonly text: string;
};

export type MonitorFrame = {
  readonly prompt: string;
  readonly lines: readonly MonitorToken[];
  readonly extra: readonly MonitorToken[];
  readonly jsx: readonly MonitorToken[];
};

export type FeatureCardCopy = {
  readonly key: string;
  readonly span: number;
  readonly title: string;
  readonly description: string;
};

const FEATURE_CARD_KEYS = [
  { key: 'models', span: 5 },
  { key: 'tokens', span: 3 },
  { key: 'routing', span: 4 },
  { key: 'deployment', span: 4 },
  { key: 'monitoring', span: 5 },
  { key: 'quota', span: 3 },
] as const;

export function getMonitorFrames(t: TranslateFn): readonly MonitorFrame[] {
  return [
    {
      prompt: t('features.monitor.frames.health.prompt'),
      lines: [
        { cls: 'comp', text: t('features.monitor.frames.health.line1.name') },
        { cls: 'str', text: t('features.monitor.frames.health.line1.status') },
        { cls: 'attr', text: t('features.monitor.frames.health.line1.metric') },
      ],
      jsx: [
        { cls: 'comp', text: t('features.monitor.frames.health.line2.name') },
        { cls: 'str', text: t('features.monitor.frames.health.line2.status') },
        { cls: 'attr', text: t('features.monitor.frames.health.line2.metric') },
      ],
      extra: [
        { cls: 'comp', text: t('features.monitor.frames.health.line3.name') },
        { cls: 'kw', text: t('features.monitor.frames.health.line3.status') },
        { cls: 'attr', text: t('features.monitor.frames.health.line3.metric') },
      ],
    },
    {
      prompt: t('features.monitor.frames.stats.prompt'),
      lines: [
        { cls: 'attr', text: t('features.monitor.frames.stats.line1.label') },
        { cls: 'num', text: t('features.monitor.frames.stats.line1.value') },
        { cls: 'comp', text: t('features.monitor.frames.stats.line1.suffix') },
      ],
      jsx: [
        { cls: 'attr', text: t('features.monitor.frames.stats.line2.label') },
        { cls: 'str', text: t('features.monitor.frames.stats.line2.value') },
        { cls: 'comp', text: t('features.monitor.frames.stats.line2.suffix') },
      ],
      extra: [
        { cls: 'attr', text: t('features.monitor.frames.stats.line3.label') },
        { cls: 'num', text: t('features.monitor.frames.stats.line3.value') },
        { cls: 'comp', text: t('features.monitor.frames.stats.line3.suffix') },
      ],
    },
    {
      prompt: t('features.monitor.frames.tokens.prompt'),
      lines: [
        { cls: 'comp', text: t('features.monitor.frames.tokens.line1.name') },
        { cls: 'str', text: t('features.monitor.frames.tokens.line1.status') },
        { cls: 'attr', text: t('features.monitor.frames.tokens.line1.metric') },
      ],
      jsx: [
        { cls: 'comp', text: t('features.monitor.frames.tokens.line2.name') },
        { cls: 'str', text: t('features.monitor.frames.tokens.line2.status') },
        { cls: 'attr', text: t('features.monitor.frames.tokens.line2.metric') },
      ],
      extra: [
        { cls: 'comp', text: t('features.monitor.frames.tokens.line3.name') },
        { cls: 'kw', text: t('features.monitor.frames.tokens.line3.status') },
        { cls: 'attr', text: t('features.monitor.frames.tokens.line3.metric') },
      ],
    },
  ];
}

export function getDeployOptions(t: TranslateFn) {
  return [
    { label: t('features.deployOptions.dockerCompose'), accent: 'rgba(255,255,255,0.5)' },
    { label: t('features.deployOptions.kubernetes'), accent: 'rgba(255,255,255,0.5)' },
    { label: t('features.deployOptions.binary'), accent: 'rgba(255,255,255,0.5)' },
    { label: t('features.deployOptions.sourceBuild'), accent: 'rgba(255,255,255,0.5)' },
  ] as const;
}

export function getFeatureCards(t: TranslateFn): readonly FeatureCardCopy[] {
  return FEATURE_CARD_KEYS.map((item) => ({
    key: item.key,
    span: item.span,
    title: t(`features.cards.${item.key}.title`),
    description: t(`features.cards.${item.key}.description`),
  }));
}
