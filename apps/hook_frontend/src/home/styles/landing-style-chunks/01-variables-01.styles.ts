export const landingStyles01Variables01 = String.raw`
html[data-home='true'] {
  /* Background colors */
  --bg-body: #F8FAFC;
  --bg-section-alt: #F4F6F8;
  --bg-card: rgba(255, 255, 255, 0.78);
  --bg-elevated: #FFFFFF;
  --bg-hover: rgba(105, 80, 232, 0.08);
  --bg-glass: rgba(255, 255, 255, 0.72);
  --bg-glass-hover: rgba(255, 255, 255, 0.9);
  --bg-glass-strong: rgba(255, 255, 255, 0.96);
  --bg-input: rgba(99, 115, 129, 0.04);
  --bg-code-token: rgba(105, 80, 232, 0.08);

  /* Border colors */
  --border-primary: rgba(28, 37, 46, 0.12);
  --border-secondary: rgba(99, 115, 129, 0.16);
  --border-subtle: rgba(145, 158, 171, 0.24);
  --border-strong: rgba(28, 37, 46, 0.18);

  /* Brand colors */
  --color-primary: #6950E8;
  --color-primary-hover: #3828A7;
  --color-accent: #8E33FF;
  --color-accent-muted: #B985F4;
  --color-warm: #FFAB00;

  /* Text colors */
  --text-primary: #1C252E;
  --text-secondary: #454F5B;
  --text-muted: #637381;
  --text-dimmed: #919EAB;
  --text-soft: rgba(28, 37, 46, 0.36);
  --text-link: #1C252E;

  /* Radii */
  --radius-sm: 10px;
  --radius-md: 12px;
  --radius-lg: 16px;
  --radius-xl: 24px;
  --radius-full: 50px;

  /* Shadows */
  --shadow-dropdown: 0 18px 48px rgba(28, 37, 46, 0.12);
  --shadow-primary: 0 18px 42px rgba(105, 80, 232, 0.24);
  --shadow-card: 0 24px 64px rgba(28, 37, 46, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.72);
  --shadow-card-soft: 0 14px 40px rgba(28, 37, 46, 0.08), inset 0 1px 0 rgba(255, 255, 255, 0.72);

  /* Transitions */
  --transition-fast: 0.15s ease;
  --transition-base: 0.2s ease;
  --transition-slow: 0.3s ease;

  /* Shared gradient for section titles */
  --gradient-title: linear-gradient(135deg, #1C252E 0%, #6950E8 28%, #FFAB00 50%, #8E33FF 72%, #1C252E 100%);

  /* Landing-specific theme tokens */
  --landing-title-color: var(--text-primary);
  --landing-card-overlay: linear-gradient(to top, rgba(28, 37, 46, 0.62) 0%, transparent 100%);
  --landing-hero-band-blend: multiply;
  --landing-hero-band-opacity: 0.54;
  --landing-shine-color: rgba(255, 255, 255, 0.75);
  --landing-logo-color: var(--text-primary);
  --landing-logo-filter: brightness(0) saturate(100%) invert(12%) sepia(16%) saturate(901%) hue-rotate(169deg) brightness(92%) contrast(89%);
  --landing-glow: rgba(105, 80, 232, 0.16);
  --sponsor-diamond-badge-bg: rgba(105, 80, 232, 0.1);
  --sponsor-diamond-badge-border: rgba(105, 80, 232, 0.2);
  --sponsor-diamond-badge-color: var(--color-primary);
  --sponsor-platinum-badge-color: var(--text-secondary);
  --sponsor-silver-badge-color: var(--text-muted);
  --sponsor-card-arrow: var(--text-dimmed);
  --sponsor-card-arrow-hover: var(--text-secondary);
  --sponsor-empty-bg: rgba(99, 115, 129, 0.02);
  --sponsor-empty-border: rgba(145, 158, 171, 0.28);
  --sponsor-empty-hover-border: var(--border-secondary);
  --sponsor-empty-color: var(--text-dimmed);
  --feature-variant-accent: rgba(105, 80, 232, 0.72);
  --feature-star-fill: #6950E8;
  --feature-star-stroke: rgba(105, 80, 232, 0.58);
  --demo-shape-border: rgba(105, 80, 232, 0.18);
  --demo-shape-hover: rgba(105, 80, 232, 0.08);
  --demo-ring-primary: #6950E8;
  --demo-ring-secondary: #FFAB00;
}

html[data-home='true'][data-color-scheme='dark'] {
  /* Background colors */
  --bg-body: #120F17;
  --bg-section-alt: #0E0B13;
  --bg-card: #1B1722;
  --bg-elevated: #1B1722;
  --bg-hover: #373044;
  --bg-glass: rgba(18, 15, 23, 0.45);
  --bg-glass-hover: rgba(18, 15, 23, 0.55);
  --bg-glass-strong: rgba(18, 15, 23, 0.95);
  --bg-input: rgba(255, 255, 255, 0.01);
  --bg-code-token: rgba(218, 209, 246, 0.08);

  /* Border colors */
  --border-primary: #2F293A;
  --border-secondary: #2F293A;
  --border-subtle: #2F293A;
  --border-strong: rgba(255, 255, 255, 0.15);

  /* Brand colors */
  --color-primary: #A855F7;
  --color-primary-hover: #7C3AED;
  --color-accent: #B497CF;
  --color-accent-muted: #B497CF;
  --color-warm: #FFAB00;

  /* Text colors */
  --text-primary: #FFFFFF;
  --text-secondary: #F5F5F5;
  --text-muted: #AAAAAA;
  --text-dimmed: #A1A1AA;
  --text-soft: rgba(255, 255, 255, 0.35);
  --text-link: #FFFFFF;

  /* Shadows */
  --shadow-dropdown: 0 8px 32px rgba(0, 0, 0, 0.4);
  --shadow-primary: 0 2px 8px rgba(82, 39, 255, 0.4);
  --shadow-card: 0 4px 32px rgba(0, 0, 0, 0.25), inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
  --shadow-card-soft: 0 2px 16px rgba(0, 0, 0, 0.2), inset 0 0.5px 0 rgba(255, 255, 255, 0.06);

  /* Shared gradient for section titles */
  --gradient-title: linear-gradient(135deg, #FFFFFF 0%, #8660FA 20%, #A855F7 40%, #8400FF 60%, #A855F7 80%, #FFFFFF 100%);

  /* Landing-specific theme tokens */
  --landing-title-color: var(--text-primary);
  --landing-card-overlay: linear-gradient(to top, rgba(10, 8, 14, 0.75) 0%, transparent 100%);
  --landing-hero-band-blend: screen;
  --landing-hero-band-opacity: 1;
  --landing-shine-color: rgba(255, 255, 255, 0.4);
  --landing-logo-color: var(--text-primary);
  --landing-logo-filter: none;
  --landing-glow: rgba(168, 85, 247, 0.18);
  --sponsor-diamond-badge-bg: rgba(168, 85, 247, 0.1);
  --sponsor-diamond-badge-border: rgba(168, 85, 247, 0.2);
  --sponsor-diamond-badge-color: #C084FC;
  --sponsor-platinum-badge-color: rgba(255, 255, 255, 0.6);
  --sponsor-silver-badge-color: rgba(255, 255, 255, 0.5);
  --sponsor-card-arrow: rgba(255, 255, 255, 0.25);
  --sponsor-card-arrow-hover: rgba(255, 255, 255, 0.6);
  --sponsor-empty-bg: transparent;
  --sponsor-empty-border: rgba(255, 255, 255, 0.06);
  --sponsor-empty-hover-border: rgba(255, 255, 255, 0.06);
  --sponsor-empty-color: rgba(255, 255, 255, 0.15);
  --feature-variant-accent: rgba(255, 255, 255, 0.5);
  --feature-star-fill: #FFFFFF;
  --feature-star-stroke: rgba(255, 255, 255, 0.5);
  --demo-shape-border: rgba(255, 255, 255, 0.08);
  --demo-shape-hover: rgba(255, 255, 255, 0.06);
  --demo-ring-primary: #FFFFFF;
  --demo-ring-secondary: #4D4D4D;
}

/* Shared keyframes — used by section titles across multiple components */
@keyframes gradientShift {
  0%,
  100% {
    background-position: 0% 50%;
  }
  50% {
    background-position: 100% 50%;
  }
}
`;
