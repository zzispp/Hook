import { useTranslate } from 'src/locales';

type ScrollDirection = 'up' | 'down';

type Provider = {
  readonly key: string;
  readonly name: string;
  readonly accent: string;
};

const PROVIDERS: readonly Provider[] = [
  {
    key: 'openai',
    name: 'OpenAI',
    accent: '#10A37F',
  },
  {
    key: 'anthropic',
    name: 'Anthropic',
    accent: '#D4A574',
  },
  {
    key: 'google',
    name: 'Google',
    accent: '#4285F4',
  },
  {
    key: 'deepseek',
    name: 'DeepSeek',
    accent: '#4D6BFE',
  },
  {
    key: 'qwen',
    name: '通义千问',
    accent: '#6554C0',
  },
  {
    key: 'azure',
    name: 'Azure OpenAI',
    accent: '#0078D4',
  },
  {
    key: 'bedrock',
    name: 'AWS Bedrock',
    accent: '#FF9900',
  },
  {
    key: 'zhipu',
    name: '智谱 AI',
    accent: '#3F6DE0',
  },
  {
    key: 'mistral',
    name: 'Mistral',
    accent: '#F54E42',
  },
  {
    key: 'ernie',
    name: '百度文心',
    accent: '#2932E1',
  },
  {
    key: 'llama',
    name: 'Meta Llama',
    accent: '#0668E1',
  },
  {
    key: 'moonshot',
    name: 'Moonshot',
    accent: '#7C3AED',
  },
  {
    key: 'midjourney',
    name: 'Midjourney',
    accent: '#E8956A',
  },
  {
    key: 'cursor',
    name: 'Cursor',
    accent: '#00C8FF',
  },
  {
    key: 'claudeCode',
    name: 'Claude Code',
    accent: '#D4A574',
  },
  {
    key: 'suno',
    name: 'Suno',
    accent: '#1DB954',
  },
  {
    key: 'yi',
    name: '零一万物',
    accent: '#00D4AA',
  },
  {
    key: 'custom',
    name: '自定义端点',
    accent: 'rgba(255,255,255,0.35)',
  },
];

const COL_1 = PROVIDERS.slice(0, 6);
const COL_2 = PROVIDERS.slice(6, 12);
const COL_3 = PROVIDERS.slice(12, 18);

function ProviderCard({ item, siteName }: { readonly item: Provider; readonly siteName: string }) {
  const { t } = useTranslate('landing');

  return (
    <div className="ln-test-card">
      <div className="ln-test-card-head">
        <div className="ln-test-card-head-left">
          <div className="ln-eco-dot" style={{ background: item.accent }} />
          <span className="ln-eco-name">{item.name}</span>
        </div>
        <span className="ln-eco-badge">{t(`ecosystem.items.${item.key}.badge`)}</span>
      </div>
      <p className="ln-test-text">{t(`ecosystem.items.${item.key}.models`, { siteName })}</p>
    </div>
  );
}

const Column = ({
  items,
  siteName,
  direction = 'up',
}: {
  readonly items: readonly Provider[];
  readonly direction?: ScrollDirection;
  readonly siteName: string;
}) => (
  <div className="ln-test-col">
    <div className={`ln-test-col-scroll ln-test-col-scroll--${direction}`}>
      {['a', 'b', 'c'].map((prefix) => (
        <div className="ln-test-col-set" key={prefix}>
          {items.map((t) => (
            <ProviderCard key={`${prefix}-${t.name}`} item={t} siteName={siteName} />
          ))}
        </div>
      ))}
    </div>
  </div>
);

type TestimonialsProps = {
  readonly siteName: string;
};

function Testimonials({ siteName }: TestimonialsProps) {
  const { t } = useTranslate('landing');

  return (
    <section className="ln-test-section">
      <div className="ln-test-inner">
        <h2 className="ln-test-title">{t('ecosystem.title')}</h2>
        <div className="ln-test-grid">
          <Column items={COL_1} direction="up" siteName={siteName} />
          <Column items={COL_2} direction="down" siteName={siteName} />
          <Column items={COL_3} direction="up" siteName={siteName} />
        </div>
      </div>
    </section>
  );
}

export default Testimonials;
