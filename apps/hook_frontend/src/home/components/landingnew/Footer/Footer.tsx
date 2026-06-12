import type { PublicSiteInfo } from 'src/types/system-setting';

import { motion } from 'motion/react';
import { AiFillHeart } from 'react-icons/ai';

import { paths } from 'src/routes/paths';

import { Link } from 'src/home/router';
import { useTranslate } from 'src/locales';
import { Logo } from 'src/home/components/common/SVGComponents';

type FooterProps = {
  readonly siteInfo: PublicSiteInfo;
};

const Footer = ({ siteInfo }: FooterProps) => {
  const { t } = useTranslate('landing');

  return (
    <footer className="ln-footer">
      <div className="ln-footer-glow" />

      <div className="ln-footer-separator" />

      <motion.div
        className="ln-footer-inner"
        initial={{ opacity: 0, y: 24 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true, margin: '-60px' }}
        transition={{ duration: 0.5, ease: [0.21, 0.47, 0.32, 0.98] }}
      >
        <div className="ln-footer-top">
          <div className="ln-footer-brand">
            <div className="ln-footer-logo-wrap" style={{ display: 'inline-flex', color: 'var(--text-primary)' }}>
              <Logo label={siteInfo.site_name} source={siteInfo.site_logo_base64} />
            </div>
            <p className="ln-footer-tagline">{siteInfo.site_subtitle}</p>
          </div>

          <nav className="ln-footer-nav">
            <div className="ln-footer-col">
              <span className="ln-footer-col-title">{t('footer.navigation')}</span>
              <Link to={paths.auth.jwt.signIn} className="ln-footer-link">
                {t('footer.console')}
              </Link>
              <Link to={paths.dashboard.models} className="ln-footer-link">
                {t('footer.catalog')}
              </Link>
              <Link to={paths.dashboard.modelStatus} className="ln-footer-link">
                {t('footer.status')}
              </Link>
            </div>

            <div className="ln-footer-col">
              <span className="ln-footer-col-title">{t('footer.community')}</span>
              <a href="https://github.com/zzispp/Hook" target="_blank" rel="noopener noreferrer" className="ln-footer-link">
                GitHub
              </a>
            </div>
          </nav>
        </div>

        <div className="ln-footer-bottom">
          <p className="ln-footer-attribution">
            {t('footer.createdWith')}{' '}
            <AiFillHeart className="ln-footer-heart" style={{ color: 'var(--color-primary)' }} />{' '}
            {t('footer.by')}{' '}
            <a href="https://github.com/zzispp" target="_blank" rel="noopener noreferrer" className="ln-footer-creator">
              zzispp
            </a>
          </p>
          <p className="ln-footer-copy">© {new Date().getFullYear()} {siteInfo.site_name}</p>
        </div>
      </motion.div>
    </footer>
  );
};

export default Footer;
