'use client';

import type { PublicSiteInfo } from 'src/types/system-setting';

import { useMemo, useState, useEffect, useCallback, useLayoutEffect } from 'react';

import { useTranslate } from 'src/locales';
import { useSiteInfo } from 'src/actions/system-settings';
import { normalizePublicSiteInfo } from 'src/actions/site-info-utils';

import CTA from '../components/landingnew/CTA/CTA';
import useScrollToTop from '../hooks/useScrollToTop';
import Hero from '../components/landingnew/Hero/Hero';
import { LandingStyles } from '../styles/LandingStyles';
import Navbar from '../components/landingnew/Navbar/Navbar';
import Footer from '../components/landingnew/Footer/Footer';
import Features from '../components/landingnew/Features/Features';
import LiveDemo from '../components/landingnew/LiveDemo/LiveDemo';
import Sponsors from '../components/landingnew/Sponsors/Sponsors';
import QuickStart from '../components/landingnew/QuickStart/QuickStart';
import Testimonials from '../components/landingnew/Testimonials/Testimonials';
import LandingLoader from '../components/landingnew/LandingLoader/LandingLoader';

const MIN_LOADER_MS = 800;
const HOME_BACKGROUND_ATTRIBUTE = 'data-home';

function useHomeDocumentBackground() {
  useLayoutEffect(() => {
    document.documentElement.setAttribute(HOME_BACKGROUND_ATTRIBUTE, 'true');

    return () => {
      document.documentElement.removeAttribute(HOME_BACKGROUND_ATTRIBUTE);
    };
  }, []);
}

function LandingSiteInfoLoading() {
  const { t } = useTranslate('landing');

  return (
    <section className="ln-site-status" aria-busy="true">
      <div className="ln-site-status-mark" />
      <p className="ln-site-status-title">{t('siteInfoStatus.loading')}</p>
    </section>
  );
}

function LandingSiteInfoError({ error, onRetry }: { readonly error: Error; readonly onRetry: () => void }) {
  const { t } = useTranslate('landing');

  return (
    <section className="ln-site-status ln-site-status-error">
      <p className="ln-site-status-title">{t('siteInfoStatus.errorTitle')}</p>
      <p className="ln-site-status-message">{error.message}</p>
      <button className="ln-site-status-action" type="button" onClick={onRetry}>
        {t('siteInfoStatus.retry')}
      </button>
    </section>
  );
}

function LandingContent({ loaded, siteInfo }: { readonly loaded: boolean; readonly siteInfo: PublicSiteInfo }) {
  return (
    <>
      <Navbar siteInfo={siteInfo} />
      <Hero visualReady={loaded} siteInfo={siteInfo} />
      <Features siteName={siteInfo.site_name} />
      <Testimonials siteName={siteInfo.site_name} />
      <LiveDemo siteName={siteInfo.site_name} />
      <QuickStart />
      <Sponsors />
      <CTA siteName={siteInfo.site_name} />
      <Footer siteInfo={siteInfo} />
    </>
  );
}

const LandingPage = () => {
  useScrollToTop();
  useHomeDocumentBackground();
  const { t } = useTranslate('landing');
  const site = useSiteInfo();
  const [loaded, setLoaded] = useState(false);
  const [hiding, setHiding] = useState(false);
  const siteInfo = useMemo(() => normalizePublicSiteInfo(site.data), [site.data]);
  const siteError = useMemo(() => {
    if (site.error) {
      return site.error;
    }

    if (site.data && !siteInfo) {
      return new Error(t('siteInfoStatus.invalidSiteName'));
    }

    return undefined;
  }, [site.data, site.error, siteInfo, t]);

  const reveal = useCallback(() => {
    setHiding(true);
    // after the loader fade-out finishes, mark fully loaded & unlock scroll
    setTimeout(() => {
      setLoaded(true);
    }, 600);
  }, []);

  useLayoutEffect(() => {
    if (loaded) return undefined;
    document.documentElement.style.overflow = 'hidden';
    document.body.style.overflow = 'hidden';
    return () => {
      document.documentElement.style.overflow = '';
      document.body.style.overflow = '';
    };
  }, [loaded]);

  useEffect(() => {
    const start = Date.now();

    document.fonts.ready.then(() => {
      const elapsed = Date.now() - start;
      const remaining = Math.max(0, MIN_LOADER_MS - elapsed);
      setTimeout(reveal, remaining);
    });
  }, [reveal]);

  return (
    <>
      <LandingStyles />
      {!loaded && <LandingLoader hiding={hiding} />}
      <section className={`landing-wrapper no-side-fades${loaded ? ' ln-loaded' : ' ln-loading'}`}>
        {siteError && <LandingSiteInfoError error={siteError} onRetry={() => void site.refresh()} />}
        {!siteError && !siteInfo && <LandingSiteInfoLoading />}
        {!siteError && siteInfo && <LandingContent loaded={loaded} siteInfo={siteInfo} />}
      </section>
    </>
  );
};

export default LandingPage;
