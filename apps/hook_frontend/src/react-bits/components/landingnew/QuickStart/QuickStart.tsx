import { FiCopy, FiCheck } from 'react-icons/fi';
import { useRef, useState, useCallback } from 'react';

import { useTranslate } from 'src/locales';

type DeployMethod = 'docker-run' | 'docker-compose' | 'source-build';

const METHODS: readonly DeployMethod[] = ['docker-run', 'docker-compose', 'source-build'];

const COMMANDS: Readonly<Record<DeployMethod, string>> = {
  'docker-run': `docker run -d --name hook \\
  -p 5555:5555 \\
  -e DATABASE_URL="postgresql://postgres:123456@localhost:5432/hook" \\
  -e REDIS_URL="redis://localhost:6379" \\
  zzispp/hook:latest`,
  'docker-compose': `curl -fsSL https://raw.githubusercontent.com/zzispp/Hook/main/docker-compose.yml -o docker-compose.yml && docker compose up -d`,
  'source-build': `git clone https://github.com/zzispp/Hook.git && cd Hook
pnpm install
just backend-migration "up"
just run-backend`,
};

const QuickStart = () => {
  const { t } = useTranslate('landing');
  const [method, setMethod] = useState<DeployMethod>('docker-run');
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const methodLabels: Readonly<Record<DeployMethod, string>> = {
    'docker-run': t('quickStart.methods.dockerRun'),
    'docker-compose': t('quickStart.methods.dockerCompose'),
    'source-build': t('quickStart.methods.sourceBuild'),
  };

  const command = COMMANDS[method];

  const copy = useCallback(() => {
    navigator.clipboard.writeText(command);
    setCopied(true);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setCopied(false), 2000);
  }, [command]);

  return (
    <section id="quick-start" className="ln-qs-section">
      <div className="ln-qs-inner">
        <div className="ln-qs-header">
          <h2 className="ln-qs-title">{t('quickStart.title')}</h2>
        </div>

        <div className="ln-qs-terminal-wrap">
          <div className="ln-qs-glow" />
          <div className="ln-qs-terminal">
            {/* tab bar */}
            <div className="ln-qs-tab-bar">
              <div className="ln-qs-tabs">
                {METHODS.map((m) => (
                  <button
                    key={m}
                    className={`ln-qs-tab${method === m ? ' ln-qs-tab--active' : ''}`}
                    onClick={() => setMethod(m)}
                  >
                    {methodLabels[m]}
                  </button>
                ))}
              </div>
            </div>

            {/* command line */}
            <div className="ln-qs-cmd-area">
              <div className="ln-qs-cmd-line">
                <span className="ln-qs-prompt">~</span>
                <code className="ln-qs-cmd-text" style={{ whiteSpace: 'pre-wrap' }}>
                  {command}
                </code>
              </div>
              <button
                className={`ln-qs-copy${copied ? ' ln-qs-copy--done' : ''}`}
                onClick={copy}
                aria-label={t('quickStart.copyCommand')}
              >
                {copied ? <FiCheck size={14} /> : <FiCopy size={14} />}
              </button>
            </div>
          </div>
          <p className="ln-qs-hint">{t('quickStart.hint')}</p>
        </div>
      </div>
    </section>
  );
};

export default QuickStart;
