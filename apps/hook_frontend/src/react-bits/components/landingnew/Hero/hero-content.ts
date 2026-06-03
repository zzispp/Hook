export type HeroCodeSnippet = {
  readonly label: string;
  readonly code: (apiBaseUrl: string) => string;
};

type HeroTranslator = (key: string) => string;

export function getHeroCodeSnippets(t: HeroTranslator): readonly HeroCodeSnippet[] {
  const prompt = t('hero.code.prompt');

  return [
    {
      label: 'cURL',
      code: (apiBaseUrl) => `curl "${apiBaseUrl}/chat/completions" \\
  -H "Authorization: Bearer $HOOK_API_KEY" \\
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
  apiKey: process.env.HOOK_API_KEY,
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
    api_key=os.environ["HOOK_API_KEY"],
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
