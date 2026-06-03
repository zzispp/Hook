'use client';

import { useState, useEffect, useCallback, useLayoutEffect } from 'react';

import CTA from '../components/landingnew/CTA/CTA';
import useScrollToTop from '../hooks/useScrollToTop';
import Hero from '../components/landingnew/Hero/Hero';
import Navbar from '../components/landingnew/Navbar/Navbar';
import Footer from '../components/landingnew/Footer/Footer';
import Features from '../components/landingnew/Features/Features';
import LiveDemo from '../components/landingnew/LiveDemo/LiveDemo';
import Sponsors from '../components/landingnew/Sponsors/Sponsors';
import QuickStart from '../components/landingnew/QuickStart/QuickStart';
import Testimonials from '../components/landingnew/Testimonials/Testimonials';
import LandingLoader from '../components/landingnew/LandingLoader/LandingLoader';

const MIN_LOADER_MS = 800;
const HERO_DEBUG_PREFIX = '[hero-bg]';
const HOME_BACKGROUND_ATTRIBUTE = 'data-react-bits-home';

function debugHeroBg(event: string, payload: Record<string, unknown> = {}) {
  console.log(`${HERO_DEBUG_PREFIX} ${JSON.stringify({ event, ...payload })}`);
}

function useHomeDocumentBackground() {
  useLayoutEffect(() => {
    document.documentElement.setAttribute(HOME_BACKGROUND_ATTRIBUTE, 'true');

    return () => {
      document.documentElement.removeAttribute(HOME_BACKGROUND_ATTRIBUTE);
    };
  }, []);
}

const LandingPage = () => {
  useScrollToTop();
  useHomeDocumentBackground();
  const [loaded, setLoaded] = useState(false);
  const [hiding, setHiding] = useState(false);

  const reveal = useCallback(() => {
    debugHeroBg('LandingPage reveal start');
    setHiding(true);
    // after the loader fade-out finishes, mark fully loaded & unlock scroll
    setTimeout(() => {
      debugHeroBg('LandingPage loaded true');
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
      debugHeroBg('LandingPage fonts ready', { elapsed, remaining });
      setTimeout(reveal, remaining);
    });
  }, [reveal]);

  return (
    <>
      {!loaded && <LandingLoader hiding={hiding} />}
      <section className={`landing-wrapper no-side-fades${loaded ? ' ln-loaded' : ' ln-loading'}`}>
        <Navbar />
        <Hero visualReady={loaded} />
        <Features />
        <Testimonials />
        <LiveDemo />
        <QuickStart />
        <Sponsors />
        <CTA />
        <Footer />
      </section>
    </>
  );
};

export default LandingPage;
