export const MODEL_STATUS_API_FORMATS = [
  'openai:chat',
  'openai:cli',
  'claude:chat',
  'gemini:chat',
] as const;

export const MODEL_STATUS_INTERVALS = [
  { label: '1m', value: 60 },
  { label: '5m', value: 300 },
  { label: '15m', value: 900 },
  { label: '30m', value: 1_800 },
  { label: '1h', value: 3_600 },
];
