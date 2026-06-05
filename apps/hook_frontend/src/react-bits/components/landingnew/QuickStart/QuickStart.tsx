import { FiCopy, FiCheck } from 'react-icons/fi';
import { useRef, useState, useCallback } from 'react';

import { useTranslate } from 'src/locales';

type DeployMethod = 'docker-compose' | 'source-build';

const METHODS: readonly DeployMethod[] = ['docker-compose', 'source-build'];

const COMMANDS: Readonly<Record<DeployMethod, string>> = {
  'docker-compose': `git clone https://github.com/zzispp/Hook.git && cd Hook
./deploy.sh`,
  'source-build': `git clone https://github.com/zzispp/Hook.git && cd Hook
pnpm install
cp config/config.yaml config.yaml
scripts/generate-password-hash.sh "your-password"
# update config.yaml before running migrations
cargo run -p hook_backend -- migration up
pnpm build:frontend:embedded
cargo run -p hook_backend`,
};

const QuickStart = () => {
  const { t } = useTranslate('landing');
  const [method, setMethod] = useState<DeployMethod>('docker-compose');
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const methodLabels: Readonly<Record<DeployMethod, string>> = {
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
