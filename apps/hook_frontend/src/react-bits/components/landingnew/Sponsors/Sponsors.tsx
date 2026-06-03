import { motion } from 'motion/react';

import { useTranslate } from 'src/locales';

import { Iconify } from 'src/components/iconify';

type AIProvider = {
  readonly key: string;
  readonly name: string;
  readonly icon: string;
  readonly url: string;
};

const PROVIDERS: readonly AIProvider[] = [
  {
    key: 'openai',
    name: 'OpenAI',
    icon: 'logos:openai-icon',
    url: 'https://openai.com',
  },
  {
    key: 'anthropic',
    name: 'Anthropic Claude',
    icon: 'logos:anthropic-icon',
    url: 'https://anthropic.com',
  },
  {
    key: 'gemini',
    name: 'Google Gemini',
    icon: 'logos:google-gemini',
    url: 'https://deepmind.google/technologies/gemini/',
  },
  {
    key: 'deepseek',
    name: 'DeepSeek',
    icon: 'simple-icons:deepseek',
    url: 'https://deepseek.com',
  },
  {
    key: 'local',
    name: 'Local & Private',
    icon: 'simple-icons:ollama',
    url: 'https://ollama.com',
  },
  {
    key: 'midjourney',
    name: 'Midjourney',
    icon: 'solar:palette-bold',
    url: 'https://midjourney.com',
  },
  {
    key: 'qwen',
    name: 'Ali Qwen',
    icon: 'simple-icons:alibabacloud',
    url: 'https://tongyi.aliyun.com',
  },
  {
    key: 'groq',
    name: 'Groq / LLaMA',
    icon: 'simple-icons:meta',
    url: 'https://groq.com',
  },
];

const Sponsors = () => {
  const { t } = useTranslate('landing');

  return (
    <section id="providers" className="ln-sp-section">
      <div className="ln-sp-inner">
        <motion.h2
          className="ln-sp-title"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-60px' }}
          transition={{ duration: 0.5, ease: [0.21, 0.47, 0.32, 0.98] }}
        >
          {t('providers.title')}
        </motion.h2>
        <motion.p
          className="ln-sp-subtitle"
          initial={{ opacity: 0, y: 24 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true, margin: '-60px' }}
          transition={{ duration: 0.5, delay: 0.05, ease: [0.21, 0.47, 0.32, 0.98] }}
        >
          {t('providers.subtitle')}
        </motion.p>

        <div className="ln-providers-grid">
          {PROVIDERS.map((provider, idx) => (
            <motion.a
              key={provider.name}
              href={provider.url}
              target="_blank"
              rel="noopener noreferrer"
              className="ln-prov-card"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.5, delay: idx * 0.05, ease: [0.21, 0.47, 0.32, 0.98] }}
            >
              <div className="ln-prov-icon-wrap">
                <Iconify
                  icon={provider.icon as any}
                  width={22}
                  className={provider.name.includes('Anthropic') ? 'ln-prov-icon--dark-invert' : ''}
                  style={{ color: provider.name === 'DeepSeek' ? '#4D6BFE' : undefined }}
                />
              </div>
              <div className="ln-prov-details">
                <div className="ln-prov-name-row">
                  <span className="ln-prov-name">{provider.name}</span>
                  <span className="ln-prov-tag">{t(`providers.items.${provider.key}.tag`)}</span>
                </div>
                <span className="ln-prov-desc" title={t(`providers.items.${provider.key}.description`)}>
                  {t(`providers.items.${provider.key}.description`)}
                </span>
              </div>
            </motion.a>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Sponsors;
