export const landingStyles13LandingTheme01 = String.raw`
html[data-home='true'],
html[data-home='true'] body {
  background: var(--bg-body);
}

.landing-wrapper {
  background:
    radial-gradient(circle at 16% 12%, var(--landing-glow) 0%, transparent 34%),
    radial-gradient(circle at 82% 8%, rgba(255, 171, 0, 0.1) 0%, transparent 30%),
    var(--bg-body);
}

[data-color-scheme='dark'] .landing-wrapper {
  background: var(--bg-body);
}

.ln-features-section,
.ln-test-section,
.ln-sp-section,
.ln-cta-section,
.ln-footer {
  background: var(--bg-body);
}

.ln-demo-section,
.ln-qs-section {
  background: var(--bg-section-alt);
}

.ln-navbar {
  z-index: 1100;
}

.ln-navbar-divider,
.ln-navbar-link,
.ln-navbar-link:hover,
.ln-navbar-link-active,
.ln-navbar-browse,
.ln-navbar-github,
.ln-navbar-mobile-link:hover,
.ln-navbar-mobile-pro,
.ln-navbar-mobile-pro-arrow,
.ln-hero-headline,
.ln-features-title,
.ln-features-card-body h3,
.ln-demo-title,
.ln-test-title,
.ln-qs-title,
.ln-sp-title,
.ln-cta-headline {
  color: var(--text-primary);
}

.ln-hero-description,
.ln-features-card-body p,
.ln-test-text,
.ln-cta-sub,
.ln-footer-tagline,
.ln-footer-link,
.ln-footer-creator {
  color: var(--text-muted);
}

.ln-navbar.ln-navbar-scrolled .ln-navbar-inner,
.ln-navbar-link-highlight,
.ln-navbar-browse,
.ln-navbar-github,
.ln-navbar-hamburger,
.ln-navbar-mobile-menu,
.ln-hero-tag,
.ln-hero-code-window,
.ln-features-card,
.ln-demo-card,
.ln-test-card,
.ln-qs-terminal {
  border-color: var(--border-subtle);
  background: var(--bg-glass);
  box-shadow: var(--shadow-card);
}

.ln-navbar.ln-navbar-scrolled .ln-navbar-inner,
.ln-navbar-browse,
.ln-navbar-github,
.ln-navbar-hamburger,
.ln-navbar-mobile-menu,
.ln-hero-tag,
.ln-hero-code-window,
.ln-features-card,
.ln-demo-card,
.ln-qs-terminal {
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
}

.ln-navbar.ln-navbar-scrolled .ln-navbar-inner {
  background: var(--bg-glass);
  border-color: var(--border-subtle);
}

.ln-navbar-link-highlight {
  box-shadow: var(--shadow-card-soft);
}

.ln-navbar-settings.MuiIconButton-root {
  flex: 0 0 auto;
  width: 36px;
  height: 36px;
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
  border-radius: 10px;
  background: var(--bg-glass);
  box-shadow: var(--shadow-card);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  transition: background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}

.ln-navbar-settings.MuiIconButton-root:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
  background: var(--bg-glass-hover);
}

.ln-navbar-browse:hover,
.ln-navbar-github:hover,
.ln-navbar-hamburger:hover,
.ln-hero-tag:hover {
  border-color: var(--border-strong);
  background: var(--bg-glass-hover);
  color: var(--text-primary);
}

.ln-feat-pill,
.ln-feat-vrow,
.ln-feat-aichat-inner,
.ln-qs-copy {
  border-color: var(--border-secondary);
  background: var(--bg-input);
}

.ln-navbar-mobile-link,
.ln-hero-tag,
.ln-hero-code-dropdown-trigger,
.ln-hero-code-dropdown-item,
.ln-feat-pill,
.ln-feat-tool-box,
.ln-feat-orbit-node,
.ln-feat-orbit-center,
.ln-feat-vrow-label,
.ln-feat-aichat-title,
.ln-feat-aichat-chevron,
.ln-feat-aichat-prompt,
.ln-feat-quota-label,
.ln-demo-card-category,
.ln-demo-card-name,
.ln-eco-name,
.ln-eco-badge,
.ln-qs-tab,
.ln-qs-prompt,
.ln-qs-hint,
.ln-footer-col-title,
.ln-footer-attribution,
.ln-footer-copy {
  color: var(--text-dimmed);
}

.ln-hero-code-titlebar,
.ln-features-card-body,
.ln-feat-aichat-head,
.ln-feat-aichat-prompt-row,
.ln-qs-tab-bar {
  border-color: var(--border-secondary);
}
`;
