import type { ReactNode } from 'react';

import { motion, AnimatePresence } from 'motion/react';
import { SiOpenai, SiGoogle, SiAnthropic } from 'react-icons/si';
import { useRef, useMemo, useState, useEffect, useCallback } from 'react';
import {
  FiZap,
  FiKey,
  FiCpu,
  FiLock,
  FiUsers,
  FiGlobe,
  FiShield,
  FiServer,
  FiLayers,
  FiActivity,
  FiDatabase,
  FiGitMerge,
} from 'react-icons/fi';

import { useTranslate } from 'src/locales';

import {
  getFeatureCards,
  getDeployOptions,
  getMonitorFrames,
} from './features-content';

/* ─── 1. Model & Provider Marquee ─── */

const ROW_A = [
  { name: 'GPT-4o' },
  { name: 'Claude 4 Opus' },
  { name: 'Gemini 2.5 Pro' },
  { name: 'DeepSeek R1' },
  { name: 'Llama 4' },
  { name: 'GPT-4.1' },
  { name: 'Claude 4 Sonnet' },
  { name: 'Qwen 3' },
  { name: 'Mistral Large' },
];

const ROW_B = [
  { name: 'Gemini 2.5 Flash' },
  { name: 'Claude 3.5 Haiku' },
  { name: 'GPT-4o mini' },
  { name: 'Codex' },
  { name: 'Doubao' },
  { name: 'Moonshot' },
  { name: 'Yi Lightning' },
  { name: 'GLM-4' },
  { name: 'Grok' },
  { name: 'DALL·E 3' },
];

const ModelMarquee = () => (
  <div className="ln-feat-marquee">
    <div className="ln-feat-marquee-track">
      <div className="ln-feat-marquee-scroll">
        {[...ROW_A, ...ROW_A].map((c, i) => (
          <span key={i} className="ln-feat-pill">
            {c.name}
          </span>
        ))}
      </div>
    </div>
    <div className="ln-feat-marquee-track">
      <div className="ln-feat-marquee-scroll ln-feat-marquee-scroll--rev">
        {[...ROW_B, ...ROW_B].map((c, i) => (
          <span key={i} className="ln-feat-pill">
            {c.name}
          </span>
        ))}
      </div>
    </div>
  </div>
);

/* ─── 2. Load Balancing Orbit ─── */

const ProviderOrbit = () => (
  <div className="ln-feat-orbit">
    <div className="ln-feat-orbit-center">
      <FiGitMerge size={26} />
    </div>
    <div className="ln-feat-orbit-ring ln-feat-orbit-ring--1">
      <div className="ln-feat-orbit-node ln-feat-orbit-node--top">
        <SiOpenai size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--right">
        <SiAnthropic size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--bottom">
        <SiGoogle size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--left">
        <FiServer size={16} />
      </div>
    </div>
    <div className="ln-feat-orbit-ring ln-feat-orbit-ring--2">
      <div className="ln-feat-orbit-node ln-feat-orbit-node--top">
        <FiZap size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--tr">
        <FiActivity size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--right">
        <FiGlobe size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--br">
        <FiCpu size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--bottom">
        <FiDatabase size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--bl">
        <FiShield size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--left">
        <FiLayers size={16} />
      </div>
      <div className="ln-feat-orbit-node ln-feat-orbit-node--tl">
        <FiUsers size={16} />
      </div>
    </div>
  </div>
);

/* ─── 3. Cost & Quota Stats ─── */

const QuotaStats = () => {
  const { t } = useTranslate('landing');
  const [progress, setProgress] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      setProgress((p) => (p >= 100 ? 0 : p + 1));
    }, 60);
    return () => clearInterval(id);
  }, []);

  const usagePercent = Math.min(progress, 72);
  const budgetUsed = (usagePercent * 1.38).toFixed(1);

  return (
    <div className="ln-feat-quota">
      <div className="ln-feat-quota-header">
        <span className="ln-feat-quota-label">{t('features.quota.monthly')}</span>
        <span className="ln-feat-quota-value">${budgetUsed} / $100</span>
      </div>
      <div className="ln-feat-quota-bar-track">
        <motion.div
          className="ln-feat-quota-bar-fill"
          animate={{ width: `${usagePercent}%` }}
          transition={{ duration: 0.1 }}
        />
      </div>
      <div className="ln-feat-quota-rows">
        <div className="ln-feat-quota-row">
          <span className="ln-feat-quota-row-dot ln-feat-quota-row-dot--a" />
          <span className="ln-feat-quota-row-name">GPT-4o</span>
          <span className="ln-feat-quota-row-val">42.3k tokens</span>
        </div>
        <div className="ln-feat-quota-row">
          <span className="ln-feat-quota-row-dot ln-feat-quota-row-dot--b" />
          <span className="ln-feat-quota-row-name">Claude 4</span>
          <span className="ln-feat-quota-row-val">28.1k tokens</span>
        </div>
        <div className="ln-feat-quota-row">
          <span className="ln-feat-quota-row-dot ln-feat-quota-row-dot--c" />
          <span className="ln-feat-quota-row-name">Gemini</span>
          <span className="ln-feat-quota-row-val">15.7k tokens</span>
        </div>
      </div>
    </div>
  );
};

/* ─── 4. Token / Key Management ─── */

const TokenFloat = () => (
  <div className="ln-feat-tools">
    <motion.div
      className="ln-feat-tool-box ln-feat-tool-box--center"
      animate={{ y: [-4, 4, -4] }}
      transition={{ duration: 4, repeat: Infinity, ease: 'easeInOut' }}
    >
      <FiKey size={22} />
    </motion.div>
    <motion.div
      className="ln-feat-tool-box ln-feat-tool-box--left"
      animate={{ y: [3, -3, 3], x: [-3, 2, -3] }}
      transition={{ duration: 5, repeat: Infinity, ease: 'easeInOut', delay: 0.6 }}
    >
      <FiUsers size={18} />
    </motion.div>
    <motion.div
      className="ln-feat-tool-box ln-feat-tool-box--right"
      animate={{ y: [3, -3, 3], x: [3, -2, 3] }}
      transition={{ duration: 5.5, repeat: Infinity, ease: 'easeInOut', delay: 1.2 }}
    >
      <FiLock size={18} />
    </motion.div>
  </div>
);

/* ─── 5. Health Monitor Terminal ─── */

const HealthTerminal = ({ siteName }: { readonly siteName: string }) => {
  const { t } = useTranslate('landing');
  const [idx, setIdx] = useState(0);
  const [typed, setTyped] = useState('');
  const [phase, setPhase] = useState<'prompt' | 'thinking' | 'code'>('prompt');
  const [codeLines, setCodeLines] = useState(0);
  const timers = useRef<ReturnType<typeof setTimeout>[]>([]);
  const monitorFrames = useMemo(() => getMonitorFrames(t), [t]);

  const clearTimers = useCallback(() => {
    timers.current.forEach(clearTimeout);
    timers.current = [];
  }, []);

  const scheduleTimer = useCallback((fn: () => void, ms: number) => {
    const id = setTimeout(fn, ms);
    timers.current.push(id);
    return id;
  }, []);

  useEffect(() => {
    const conv = monitorFrames[idx];
    setTyped('');
    setPhase('prompt');
    setCodeLines(0);

    let delay = 300;
    for (let c = 0; c <= conv.prompt.length; c++) {
      const slice = conv.prompt.slice(0, c);
      scheduleTimer(() => setTyped(slice), delay);
      delay += 50;
    }

    scheduleTimer(() => setPhase('thinking'), delay);
    delay += 900;

    scheduleTimer(() => {
      setPhase('code');
      setCodeLines(1);
    }, delay);
    delay += 280;
    scheduleTimer(() => setCodeLines(2), delay);
    delay += 280;
    scheduleTimer(() => setCodeLines(3), delay);
    delay += 2400;

    scheduleTimer(() => setIdx((p) => (p + 1) % monitorFrames.length), delay);

    return clearTimers;
  }, [idx, clearTimers, monitorFrames, scheduleTimer]);

  const conv = monitorFrames[idx];

  return (
    <div className="ln-feat-aichat">
      <AnimatePresence mode="wait">
        <motion.div
          key={idx}
          className="ln-feat-aichat-inner"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.3 }}
        >
          <div className="ln-feat-aichat-head">
            <div className="ln-feat-aichat-dots">
              <span />
              <span />
              <span />
            </div>
            <span className="ln-feat-aichat-title">{t('features.monitor.title', { siteName })}</span>
          </div>

          <div className="ln-feat-aichat-prompt-row">
            <span className="ln-feat-aichat-chevron">$</span>
            <span className="ln-feat-aichat-prompt">{typed}</span>
            {phase === 'prompt' && <span className="ln-feat-aichat-cursor" />}
          </div>

          {phase === 'thinking' && (
            <div className="ln-feat-aichat-thinking">
              <span />
              <span />
              <span />
            </div>
          )}

          {phase === 'code' && (
            <div className="ln-feat-aichat-code-block">
              {codeLines >= 1 && (
                <motion.div
                  className="ln-feat-aichat-code-line"
                  initial={{ opacity: 0, x: -6 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2 }}
                >
                  <span className="ln-feat-aichat-ln">1</span>
                  {conv.lines.map((token, j) => (
                    <span key={j} className={`ac-${token.cls}`}>
                      {token.text}
                    </span>
                  ))}
                </motion.div>
              )}
              {codeLines >= 2 && (
                <motion.div
                  className="ln-feat-aichat-code-line"
                  initial={{ opacity: 0, x: -6 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2 }}
                >
                  <span className="ln-feat-aichat-ln">2</span>
                  {conv.jsx.map((token, j) => (
                    <span key={j} className={`ac-${token.cls}`}>
                      {token.text}
                    </span>
                  ))}
                </motion.div>
              )}
              {codeLines >= 3 && (
                <motion.div
                  className="ln-feat-aichat-code-line"
                  initial={{ opacity: 0, x: -6 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ duration: 0.2 }}
                >
                  <span className="ln-feat-aichat-ln">3</span>
                  {conv.extra.map((token, j) => (
                    <span key={j} className={`ac-${token.cls}`}>
                      {token.text}
                    </span>
                  ))}
                </motion.div>
              )}
            </div>
          )}
        </motion.div>
      </AnimatePresence>
    </div>
  );
};

/* ─── 6. Self-Hosted Deployment Options ─── */

const DeployTabs = () => {
  const { t } = useTranslate('landing');
  const [active, setActive] = useState(0);
  const deployOptions = getDeployOptions(t);

  useEffect(() => {
    const id = setInterval(() => setActive((p) => (p + 1) % 4), 2200);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="ln-feat-vrows">
      {deployOptions.map((v, i) => (
        <motion.div
          key={i}
          className="ln-feat-vrow"
          animate={{
            opacity: i === active ? 1 : 0.3,
          }}
          transition={{ duration: 0.4 }}
        >
          <div className="ln-feat-vrow-dot" style={{ background: v.accent }} />
          <span className="ln-feat-vrow-label">{v.label}</span>
          <div className="ln-feat-vrow-bars">
            <div className="ln-feat-vrow-bar" style={{ width: `${35 + i * 8}%` }} />
            <div className="ln-feat-vrow-bar" style={{ width: `${20 + ((i + 2) % 4) * 7}%` }} />
            <div className="ln-feat-vrow-bar" style={{ width: `${45 + ((i + 1) % 3) * 10}%` }} />
          </div>
        </motion.div>
      ))}
    </div>
  );
};

/* ─── Card data ─── */

type FeatureCard = {
  readonly key: string;
  readonly title: string;
  readonly desc: string;
  readonly span: number;
  readonly visual: ReactNode;
};

function getCardVisuals(siteName: string): Record<string, ReactNode> {
  return {
    models: <ModelMarquee />,
    tokens: <TokenFloat />,
    routing: <ProviderOrbit />,
    deployment: <DeployTabs />,
    monitoring: <HealthTerminal siteName={siteName} />,
    quota: <QuotaStats />,
  };
}

/* ─── Features Section ─── */

type FeaturesProps = {
  readonly siteName: string;
};

const Features = ({ siteName }: FeaturesProps) => {
  const { t } = useTranslate('landing');
  const cardVisuals = useMemo(() => getCardVisuals(siteName), [siteName]);
  const cards: readonly FeatureCard[] = getFeatureCards(t).map((card) => ({
    key: card.key,
    title: card.title,
    desc: card.description,
    span: card.span,
    visual: cardVisuals[card.key],
  }));

  return (
    <section id="features" className="ln-features-section">
      <div className="ln-features-inner">
        <h2 className="ln-features-title">{t('features.title')}</h2>

        <div className="ln-features-grid">
          {cards.map((card, i) => (
            <motion.div
              key={card.key}
              className={`ln-features-card ln-features-card--span-${card.span}`}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.5, delay: i * 0.07, ease: [0.21, 0.47, 0.32, 0.98] }}
            >
              <div className="ln-features-card-visual">{card.visual}</div>
              <div className="ln-features-card-body">
                <h3>{card.title}</h3>
                <p>{card.desc}</p>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
