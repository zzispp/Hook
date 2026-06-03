import type { MouseEvent, CSSProperties } from 'react';

import { FaGithub } from 'react-icons/fa6';
import { useRef, useState, useEffect, useCallback } from 'react';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales';
import { Link, useLocation } from 'src/react-bits/router';
import { primaryColorPresets } from 'src/theme/with-settings';
import { SettingsButton } from 'src/layouts/components/settings-button';

import { useSettingsContext } from 'src/components/settings';

import { useAuthContext } from 'src/auth/hooks';

import { Logo } from '../../common/SVGComponents';
import { GITHUB_URL } from '../../../constants/Site';

function hexToRgbChannels(hex: string): string {
  const cleanHex = hex.replace('#', '').trim();
  let r = 0;
  let g = 0;
  let b = 0;
  if (cleanHex.length === 3) {
    r = parseInt(cleanHex[0] + cleanHex[0], 16);
    g = parseInt(cleanHex[1] + cleanHex[1], 16);
    b = parseInt(cleanHex[2] + cleanHex[2], 16);
  } else if (cleanHex.length === 6) {
    r = parseInt(cleanHex.slice(0, 2), 16);
    g = parseInt(cleanHex.slice(2, 4), 16);
    b = parseInt(cleanHex.slice(4, 6), 16);
  }
  return `${r}, ${g}, ${b}`;
}

const Navbar = () => {
  const { state } = useSettingsContext();
  const primaryColor = state.primaryColor;
  const preset = primaryColorPresets[primaryColor] || primaryColorPresets.default;

  const styleOverrides = {
    '--pro-base': preset.main,
    '--pro-dark': preset.dark,
    '--pro-light': preset.light,
    '--pro-glow': hexToRgbChannels(preset.main),
    '--pro-glow-dark': hexToRgbChannels(preset.dark),
    '--pro-glow-light': hexToRgbChannels(preset.light),
  } as CSSProperties;
  const { authenticated } = useAuthContext();
  const [scrolled, setScrolled] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const linksRef = useRef<HTMLElement | null>(null);
  const highlightRef = useRef<HTMLDivElement | null>(null);

  const { t } = useTranslate('landing');
  const location = useLocation();
  const navLinks = [
    { label: t('navbar.links.features'), to: '/#features', match: 'features' },
    { label: t('navbar.links.demo'), to: '/#live-demo', match: 'live-demo' },
    { label: t('navbar.links.quickStart'), to: '/#quick-start', match: 'quick-start' },
    { label: t('navbar.links.providers'), to: '/#providers', match: 'providers' },
  ];

  const [activeSection, setActiveSection] = useState<string>('');

  useEffect(() => {
    if (location.pathname !== '/') {
      setActiveSection('');
      return undefined;
    }

    const sectionIds = ['features', 'live-demo', 'quick-start', 'providers'];

    const handleScroll = () => {
      let currentSection = '';
      for (const id of sectionIds) {
        const element = document.getElementById(id);
        if (element) {
          const rect = element.getBoundingClientRect();
          if (rect.top < window.innerHeight * 0.4 && rect.bottom > window.innerHeight * 0.2) {
            currentSection = id;
            break;
          }
        }
      }
      setActiveSection(currentSection);
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    handleScroll();

    return () => window.removeEventListener('scroll', handleScroll);
  }, [location.pathname]);

  const handleNavLinkClick = useCallback((e: MouseEvent<HTMLAnchorElement>, to: string) => {
    const hashIndex = to.indexOf('#');
    if (hashIndex !== -1) {
      const hash = to.substring(hashIndex + 1);
      const element = document.getElementById(hash);
      if (element) {
        e.preventDefault();
        element.scrollIntoView({ behavior: 'smooth', block: 'start' });
        window.history.pushState(null, '', `#${hash}`);
        setActiveSection(hash);
        setMenuOpen(false);
      }
    }
  }, []);

  const isActive = useCallback(
    (match: string) => (location.pathname === '/' ? activeSection === match : location.pathname.startsWith(match)),
    [location.pathname, activeSection]
  );

  useEffect(() => {
    if (menuOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => { document.body.style.overflow = ''; };
  }, [menuOpen]);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 50);
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  const positionHighlight = useCallback((el: Element) => {
    const highlight = highlightRef.current;
    const container = linksRef.current;
    if (!highlight || !container || !el) return;
    const linkRect = el.getBoundingClientRect();
    const containerRect = container.getBoundingClientRect();
    highlight.style.width = `${linkRect.width}px`;
    highlight.style.height = `${linkRect.height}px`;
    highlight.style.transform = `translateX(${linkRect.left - containerRect.left}px)`;
    highlight.style.opacity = '1';
  }, []);

  const getActiveEl = useCallback(() => {
    const container = linksRef.current;
    if (!container) return null;
    return container.querySelector('.ln-navbar-link-active');
  }, []);

  const handleLinkHover = useCallback((e: MouseEvent<HTMLElement>) => {
    positionHighlight(e.currentTarget);
  }, [positionHighlight]);

  const handleLinksLeave = useCallback(() => {
    const activeEl = getActiveEl();
    if (activeEl) {
      positionHighlight(activeEl);
    } else {
      const highlight = highlightRef.current;
      if (highlight) highlight.style.opacity = '0';
    }
  }, [positionHighlight, getActiveEl]);

  // Position highlight on active link on mount, route change and activeSection change
  useEffect(() => {
    requestAnimationFrame(() => {
      const activeEl = getActiveEl();
      if (activeEl) {
        positionHighlight(activeEl);
      } else {
        const highlight = highlightRef.current;
        if (highlight) highlight.style.opacity = '0';
      }
    });
  }, [location.pathname, activeSection, positionHighlight, getActiveEl]);

  return (
    <header className={`ln-navbar${scrolled ? ' ln-navbar-scrolled' : ''}`} style={styleOverrides}>
      <div className="ln-navbar-inner">
        <div className="ln-navbar-left">
          <Link to="/" className="ln-navbar-logo">
            <Logo />
          </Link>

          <span className="ln-navbar-divider">/</span>

          <nav className="ln-navbar-links" ref={linksRef} onMouseLeave={handleLinksLeave}>
            <div className="ln-navbar-link-highlight" ref={highlightRef} />
            {navLinks.map(({ label, to, match }) => (
              <Link
                key={to}
                className={`ln-navbar-link${isActive(match) ? ' ln-navbar-link-active' : ''}`}
                to={to}
                onClick={(e) => handleNavLinkClick(e, to)}
                onMouseEnter={handleLinkHover}
              >
                {label}
              </Link>
            ))}
          </nav>
        </div>

        <div className="ln-navbar-right">
          {authenticated ? (
            <>
              <Link
                to={paths.dashboard.root}
                className="ln-navbar-pro"
                onMouseMove={e => {
                  const rect = e.currentTarget.getBoundingClientRect();
                  const x = ((e.clientX - rect.left) / rect.width) * 100;
                  e.currentTarget.style.setProperty('--pro-mx', `${x}%`);
                }}
              >
                {t('navbar.actions.console')}
              </Link>
              <Link
                to={paths.dashboard.models}
                className="ln-navbar-browse"
                style={{ padding: '0 12px' }}
              >
                {t('navbar.actions.catalog')}
              </Link>
            </>
          ) : (
            <>
              <Link
                to={paths.auth.jwt.signIn}
                className="ln-navbar-pro"
                onMouseMove={e => {
                  const rect = e.currentTarget.getBoundingClientRect();
                  const x = ((e.clientX - rect.left) / rect.width) * 100;
                  e.currentTarget.style.setProperty('--pro-mx', `${x}%`);
                }}
              >
                {t('navbar.actions.signIn')}
              </Link>
              <Link
                to={paths.auth.jwt.signUp}
                className="ln-navbar-browse"
                style={{ padding: '0 12px' }}
              >
                {t('navbar.actions.signUp')}
              </Link>
            </>
          )}

          <a
            className="ln-navbar-github"
            href={GITHUB_URL}
            target="_blank"
            rel="noopener noreferrer"
          >
            <FaGithub size={16} />
          </a>

          <SettingsButton className="ln-navbar-settings" />

          <button
            className={`ln-navbar-hamburger${menuOpen ? ' open' : ''}`}
            onClick={() => setMenuOpen(o => !o)}
            aria-label={t('navbar.actions.menu')}
          >
            <span />
            <span />
            <span />
          </button>
        </div>

        {menuOpen && (
          <div className="ln-navbar-mobile-menu">
            {authenticated ? (
              <>
                <Link to={paths.dashboard.root} className="ln-navbar-mobile-pro" onClick={() => setMenuOpen(false)}>
                  <span className="ln-navbar-mobile-pro-title">{t('navbar.actions.console')}</span>
                  <span className="ln-navbar-mobile-pro-arrow">→</span>
                </Link>
                <Link to={paths.dashboard.models} className="ln-navbar-mobile-link" onClick={() => setMenuOpen(false)}>
                  {t('navbar.actions.catalog')}
                </Link>
              </>
            ) : (
              <>
                <Link to={paths.auth.jwt.signIn} className="ln-navbar-mobile-pro" onClick={() => setMenuOpen(false)}>
                  <span className="ln-navbar-mobile-pro-title">{t('navbar.actions.signIn')}</span>
                  <span className="ln-navbar-mobile-pro-arrow">→</span>
                </Link>
                <Link to={paths.auth.jwt.signUp} className="ln-navbar-mobile-link" onClick={() => setMenuOpen(false)}>
                  {t('navbar.actions.signUp')}
                </Link>
              </>
            )}
            {navLinks.map(({ label, to }) => (
              <Link key={to} className="ln-navbar-mobile-link" to={to} onClick={(e) => handleNavLinkClick(e, to)}>
                {label}
              </Link>
            ))}
            <a
              href={GITHUB_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="ln-navbar-mobile-link"
              onClick={() => setMenuOpen(false)}
            >
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: 8 }}>
                <FaGithub size={14} /> {t('navbar.actions.github')}
              </span>
            </a>
          </div>
        )}
      </div>
    </header>
  );
};

export default Navbar;
