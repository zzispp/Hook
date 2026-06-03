'use client';

import type { RefObject, CSSProperties } from 'react';
import type { ThemeColorPreset } from 'src/theme/with-settings';

import { FaArrowRight } from 'react-icons/fa6';
import { useRef, useState, useEffect, useCallback } from 'react';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales';
import { Link } from 'src/react-bits/router';
import { useSiteInfo } from 'src/actions/system-settings';
import { primaryColorPresets } from 'src/theme/with-settings';

import { useSettingsContext } from 'src/components/settings';

import { useAuthContext } from 'src/auth/hooks';

import DotField from './DotField';
import HeroBand from './HeroBand';
import { getHeroCodeSnippets, type HeroCodeSnippet } from './hero-content';

const DEFAULT_API_BASE_URL = '/v1';
const THEME_ATTRIBUTE = 'data-color-scheme';

type HeroPalette = {
  readonly accentColor: string;
  readonly accentForeground: string;
  readonly dotGradientFrom: string;
  readonly dotGradientTo: string;
};

type HeroCodeDropdownProps = {
  readonly activeSnippet: number;
  readonly dropdownOpen: boolean;
  readonly dropdownRef: RefObject<HTMLDivElement | null>;
  readonly onSelect: (index: number) => void;
  readonly onToggle: () => void;
  readonly snippetLabel: string;
  readonly snippets: readonly HeroCodeSnippet[];
};

function getActiveSnippet(snippets: readonly HeroCodeSnippet[], snippetIndex: number) {
  const snippet = snippets[snippetIndex];

  if (!snippet) {
    throw new Error(`Missing hero code snippet for index ${snippetIndex}`);
  }

  return snippet;
}

function getBrowserApiBaseUrl(): string {
  return `${window.location.protocol}//${window.location.host}/v1`;
}

function useApiBaseUrl(): string {
  const [apiBaseUrl, setApiBaseUrl] = useState(DEFAULT_API_BASE_URL);

  useEffect(() => {
    setApiBaseUrl(getBrowserApiBaseUrl());
  }, []);

  return apiBaseUrl;
}

function hexToRgba(hex: string, alpha: number): string {
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

  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function getHeroPalette(primaryColor: ThemeColorPreset, isDark: boolean): HeroPalette {
  const preset = primaryColorPresets[primaryColor] || primaryColorPresets.default;

  return {
    accentColor: preset.main,
    accentForeground: preset.contrastText,
    dotGradientFrom: hexToRgba(preset.main, isDark ? 0.3 : 0.35),
    dotGradientTo: hexToRgba(isDark ? (preset.light || preset.main) : (preset.dark || preset.main), 0.24),
  };
}

function useHeroPalette(): HeroPalette {
  const { state } = useSettingsContext();
  const [isDark, setIsDark] = useState(false);

  useEffect(() => {
    const checkDark = () => {
      setIsDark(document.documentElement.getAttribute(THEME_ATTRIBUTE) === 'dark');
    };

    checkDark();

    const observer = new MutationObserver(checkDark);
    observer.observe(document.documentElement, { attributeFilter: [THEME_ATTRIBUTE] });

    return () => observer.disconnect();
  }, []);

  return getHeroPalette(state.primaryColor, isDark);
}

function HeroCodeDropdown(props: HeroCodeDropdownProps) {
  const { activeSnippet, dropdownOpen, dropdownRef, onSelect, onToggle, snippetLabel, snippets } = props;

  return (
    <div className="ln-hero-code-dropdown" ref={dropdownRef}>
      <button className="ln-hero-code-dropdown-trigger" onClick={onToggle}>
        {snippetLabel}
        <svg
          className={`ln-hero-code-caret${dropdownOpen ? ' open' : ''}`}
          width="8"
          height="5"
          viewBox="0 0 8 5"
          fill="none"
        >
          <path d="M1 1L4 4L7 1" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" />
        </svg>
      </button>

      <div className={`ln-hero-code-dropdown-menu${dropdownOpen ? ' open' : ''}`}>
        {snippets.map((item, index) => (
          <button
            key={item.label}
            className={`ln-hero-code-dropdown-item${index === activeSnippet ? ' active' : ''}`}
            onClick={() => onSelect(index)}
          >
            {item.label}
          </button>
        ))}
      </div>
    </div>
  );
}

function HeroCodeWindow() {
  const { t } = useTranslate('landing');
  const [activeSnippet, setActiveSnippet] = useState(0);
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement | null>(null);
  const apiBaseUrl = useApiBaseUrl();
  const snippets = getHeroCodeSnippets(t);
  const snippet = getActiveSnippet(snippets, activeSnippet);

  const toggleDropdown = useCallback(() => setDropdownOpen((prev) => !prev), []);
  const selectSnippet = useCallback((index: number) => {
    setActiveSnippet(index);
    setDropdownOpen(false);
  }, []);

  useEffect(() => {
    const onClickOutside = (event: PointerEvent) => {
      if (event.target instanceof Node && dropdownRef.current?.contains(event.target)) {
        return;
      }

      setDropdownOpen(false);
    };

    document.addEventListener('pointerdown', onClickOutside);

    return () => document.removeEventListener('pointerdown', onClickOutside);
  }, []);

  return (
    <div className="ln-hero-code-window">
      <div className="ln-hero-code-titlebar">
        <div className="ln-hero-code-dots">
          <span />
          <span />
          <span />
        </div>

        <div className="ln-hero-code-titlebar-actions">
          <HeroCodeDropdown
            activeSnippet={activeSnippet}
            dropdownOpen={dropdownOpen}
            dropdownRef={dropdownRef}
            onSelect={selectSnippet}
            onToggle={toggleDropdown}
            snippetLabel={snippet.label}
            snippets={snippets}
          />
        </div>
      </div>

      <div className="ln-hero-code-body">
        <pre className="ln-hero-code-pre">
          <code>{snippet.code(apiBaseUrl)}</code>
        </pre>
      </div>

      <p className="ln-hero-code-hint">
        {t('hero.code.baseUrl')}: {apiBaseUrl}
      </p>
    </div>
  );
}

function HeroCopy({ palette }: { readonly palette: HeroPalette }) {
  const { t } = useTranslate('landing');
  const site = useSiteInfo();
  const { authenticated } = useAuthContext();

  return (
    <div className="ln-hero-left">
      <Link to={paths.dashboard.modelStatus} className="ln-hero-tag">
        <span
          className="ln-hero-tag-new"
          style={{ background: palette.accentColor, color: palette.accentForeground }}
        >
          {t('hero.badge')}
        </span>
        {t('hero.badgeSuffix')} <FaArrowRight size={10} />
      </Link>

      <h1 className="ln-hero-headline">
        <span className="ln-hero-headline-line">{site.data?.site_name || t('hero.headlineDefaultName')}</span>
        <br />
        <span className="ln-hero-headline-line">{t('hero.headlineSuffix')}</span>
      </h1>

      <p className="ln-hero-description">{site.data?.site_subtitle || t('hero.description')}</p>

      <div className="ln-hero-buttons">
        {authenticated ? (
          <>
            <Link
              to={paths.dashboard.root}
              className="ln-hero-btn ln-hero-btn-primary"
              style={{
                background: palette.accentColor,
                borderColor: palette.accentColor,
                color: palette.accentForeground,
              }}
            >
              {t('hero.buttons.console')}
            </Link>
            <Link to={paths.dashboard.models} className="ln-hero-btn ln-hero-btn-secondary">
              {t('hero.buttons.catalog')}
            </Link>
          </>
        ) : (
          <>
            <Link
              to={paths.auth.jwt.signIn}
              className="ln-hero-btn ln-hero-btn-primary"
              style={{
                background: palette.accentColor,
                borderColor: palette.accentColor,
                color: palette.accentForeground,
              }}
            >
              {t('hero.buttons.signIn')}
            </Link>
            <Link to={paths.auth.jwt.signUp} className="ln-hero-btn ln-hero-btn-secondary">
              {t('hero.buttons.signUp')}
            </Link>
          </>
        )}
      </div>
    </div>
  );
}

function HeroBackgroundFade() {
  return (
    <svg className="ln-hero-bottom-fade" preserveAspectRatio="none" viewBox="0 0 1 1">
      <defs>
        <linearGradient id="hero-bottom-fade" x1="0" x2="0" y1="0" y2="1">
          <stop offset="0%" stopColor="#120F17" stopOpacity="0" />
          <stop offset="50%" stopColor="#120F17" stopOpacity="0" />
          <stop offset="60%" stopColor="#120F17" stopOpacity="0.03" />
          <stop offset="68%" stopColor="#120F17" stopOpacity="0.1" />
          <stop offset="74%" stopColor="#120F17" stopOpacity="0.22" />
          <stop offset="80%" stopColor="#120F17" stopOpacity="0.38" />
          <stop offset="85%" stopColor="#120F17" stopOpacity="0.55" />
          <stop offset="90%" stopColor="#120F17" stopOpacity="0.72" />
          <stop offset="94%" stopColor="#120F17" stopOpacity="0.87" />
          <stop offset="97%" stopColor="#120F17" stopOpacity="0.95" />
          <stop offset="100%" stopColor="#120F17" stopOpacity="1" />
        </linearGradient>
      </defs>
      <rect width="1" height="1" fill="url(#hero-bottom-fade)" />
    </svg>
  );
}

type HeroProps = {
  readonly visualReady: boolean;
};

function HeroVisuals({ palette, visualReady }: { readonly palette: HeroPalette; readonly visualReady: boolean }) {
  return (
    <>
      {visualReady && (
        <>
          <DotField
            className="ln-hero-dot-field"
            dotRadius={1.5}
            dotSpacing={14}
            cursorRadius={500}
            cursorForce={0.1}
            bulgeOnly
            bulgeStrength={67}
            glowRadius={160}
            sparkle={false}
            waveAmplitude={0}
            gradientFrom={palette.dotGradientFrom}
            gradientTo={palette.dotGradientTo}
          />
          <HeroBand
            className="ln-hero-band"
            color={palette.accentColor}
            speed={0.2}
            frequency={1}
            noise={0.15}
            bandWidth={0.14}
            rotation={90}
            fadeTop={0.75}
            iterations={1}
            intensity={1.25}
            scale={1}
            warpStrength={1}
            yOffset={0.3}
            mouseInfluence={0.3}
          />
        </>
      )}
      <HeroBackgroundFade />
    </>
  );
}

export default function Hero({ visualReady }: HeroProps) {
  const palette = useHeroPalette();
  const heroStyle = {
    '--ln-hero-accent': palette.accentColor,
    '--ln-hero-dot-from': palette.dotGradientFrom,
    '--ln-hero-dot-to': palette.dotGradientTo,
  } as CSSProperties;

  return (
    <section className="ln-hero" style={heroStyle}>
      <HeroVisuals palette={palette} visualReady={visualReady} />
      <div className="ln-hero-content">
        <HeroCopy palette={palette} />
        <div className="ln-hero-right">
          <HeroCodeWindow />
        </div>
      </div>
    </section>
  );
}
