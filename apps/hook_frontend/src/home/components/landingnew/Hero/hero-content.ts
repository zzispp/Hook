export type HeroCodeSnippet = {
  readonly label: string;
  readonly code: (apiBaseUrl: string) => string;
};

type HeroTranslator = (key: string, options?: Record<string, unknown>) => string;

export function getHeroCodeSnippets(t: HeroTranslator, siteName: string): readonly HeroCodeSnippet[] {
  const prompt = t('hero.code.prompt', { siteName });
  const apiKeyEnvName = getApiKeyEnvName(siteName);

  return [
    {
      label: 'cURL',
      code: (apiBaseUrl) => `curl "${apiBaseUrl}/chat/completions" \\
  -H "Authorization: Bearer $${apiKeyEnvName}" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-4o-mini",
    "messages": [
      {
        "role": "user",
        "content": "${prompt}"
      }
    ]
  }'`,
    },
    {
      label: 'Node.js',
      code: (apiBaseUrl) => `import OpenAI from 'openai';

const client = new OpenAI({
  apiKey: process.env.${apiKeyEnvName},
  baseURL: '${apiBaseUrl}',
});

const response = await client.chat.completions.create({
  model: 'gpt-4o-mini',
  messages: [
    {
      role: 'user',
      content: '${prompt}',
    },
  ],
});

console.log(response.choices[0].message.content);`,
    },
    {
      label: 'Python',
      code: (apiBaseUrl) => `import os
from openai import OpenAI

client = OpenAI(
    api_key=os.environ["${apiKeyEnvName}"],
    base_url="${apiBaseUrl}",
)

response = client.chat.completions.create(
    model="gpt-4o-mini",
    messages=[
        {
            "role": "user",
            "content": "${prompt}",
        },
    ],
)

print(response.choices[0].message.content)`,
    },
  ];
}

function getApiKeyEnvName(siteName: string): string {
  const normalized = siteName
    .trim()
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toUpperCase()
    .replace(/[^A-Z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');

  if (normalized) {
    return `${normalized}_API_KEY`;
  }

  const codePointSuffix = Array.from(siteName.trim())
    .map((char) => char.codePointAt(0)?.toString(16).toUpperCase())
    .filter(Boolean)
    .join('_');

  return `SITE_${codePointSuffix}_API_KEY`;
}
