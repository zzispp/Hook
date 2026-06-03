import { motion } from 'motion/react';
import { FaGithub } from 'react-icons/fa6';
import { FiArrowRight } from 'react-icons/fi';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales';
import { Link } from 'src/react-bits/router';

const CTA = () => {
  const { t } = useTranslate('landing');

  return (
    <section className="ln-cta-section">
      <div className="ln-cta-glow" />

      <motion.div
        className="ln-cta-inner"
        initial={{ opacity: 0, y: 24 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: '-60px' }}
        transition={{ duration: 0.5, ease: [0.21, 0.47, 0.32, 0.98] }}
      >
        <div className="ln-cta-card-wrapper">
          <div className="ln-cta-card-border" />
          <div className="ln-cta-card">
            <h2 className="ln-cta-headline">{t('cta.title')}</h2>

            <p className="ln-cta-sub">{t('cta.description')}</p>

            <div className="ln-cta-buttons">
              <Link to={paths.auth.jwt.signIn} className="ln-cta-btn ln-cta-btn--primary">
                {t('cta.primaryAction')} <FiArrowRight size={15} />
              </Link>
              <a
                href="https://github.com/zzispp/Hook"
                target="_blank"
                rel="noopener noreferrer"
                className="ln-cta-btn ln-cta-btn--secondary"
              >
                <FaGithub size={15} /> {t('cta.secondaryAction')}
              </a>
            </div>
          </div>
        </div>
      </motion.div>
    </section>
  );
};

export default CTA;
