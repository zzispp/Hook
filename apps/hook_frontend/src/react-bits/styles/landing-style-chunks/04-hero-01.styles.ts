export const landingStyles04Hero01 = String.raw`
.ln-hero {
  position: relative;
  width: 100%;
  min-height: 100vh;
  overflow: hidden;
  isolation: isolate;
  background: var(--bg-body);
}

.ln-hero::before,
.ln-hero::after {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.ln-hero::before {
  z-index: 0;
  background:
    radial-gradient(circle at 17% 55%, color-mix(in srgb, var(--ln-hero-accent) 38%, transparent) 0%, transparent 28%),
    linear-gradient(135deg, transparent 32%, color-mix(in srgb, var(--ln-hero-accent) 44%, transparent) 54%, transparent 78%),
    radial-gradient(circle at 76% 70%, color-mix(in srgb, var(--ln-hero-accent) 34%, transparent) 0%, transparent 32%);
}

.ln-hero::after {
  z-index: 0;
  opacity: 0.45;
  background-image: radial-gradient(circle, var(--ln-hero-dot-from) 0 1px, transparent 1px);
  background-size: 16px 16px;
}

.ln-hero-dot-field {
  z-index: 1;
  transform: translateZ(0);
  will-change: transform;
}

.ln-hero-band {
  position: absolute;
  mix-blend-mode: normal;
  bottom: 0;
  left: 0;
  right: 0;
  height: 150%;
  pointer-events: none;
  z-index: 2;
  transform: translateZ(0);
  will-change: transform;
}

.ln-hero-bottom-fade {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  width: 100%;
  height: min(48vh, 420px);
  display: block;
  pointer-events: none;
  z-index: 3;
}

.ln-hero-content {
  position: relative;
  z-index: 4;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  align-items: start;
  max-width: 1324px;
  margin: 0 auto;
  padding: clamp(120px, 16vw, 240px) 24px 80px;
  overflow: hidden;
}

.ln-hero-left {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.ln-hero-tag {
  display: flex;
  align-items: center;
  gap: 10px;
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0;
  color: rgba(255, 255, 255, 0.7);
  width: fit-content;
  padding: 5px 16px 5px 5px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(18, 15, 23, 0.45);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.25), inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
  transition: background 0.2s ease, color 0.2s ease, border-color 0.2s ease;
  text-decoration: none;
}

.ln-hero-tag:hover {
  background: rgba(18, 15, 23, 0.55);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.95);
}

.ln-hero-tag-new {
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  background: #A855F7;
  padding: 4px 10px;
  border-radius: 6px;
}

.ln-hero-headline {
  font-family: 'Geist Pixel Line', sans-serif;
  font-size: clamp(28px, 5.5vw, 68px);
  font-weight: 500;
  line-height: 1.1;
  letter-spacing: 0;
  color: #fff;
  margin: 0;
}

.ln-hero-headline-line {
  white-space: nowrap;
}

.ln-hero-description {
  font-family: 'Geist', sans-serif;
  font-size: 16px;
  font-weight: 400;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.606);
  margin: 0;
  max-width: 42ch;
}

.ln-hero-buttons {
  display: flex;
  gap: 12px;
  margin-top: 4px;
}

.ln-hero-btn {
  font-family: 'Geist Mono', monospace;
  font-size: 14px;
  font-weight: 500;
  padding: 12px 24px;
  border-radius: 12px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
  transition: filter var(--transition-fast);
}

.ln-hero-btn-primary {
  background: #A855F7;
  color: #fff;
  border: 1px solid #A855F7;
}

.ln-hero-btn-primary:hover {
  filter: brightness(0.85);
}

.ln-hero-btn-secondary {
  background: transparent;
  color: #fff;
  border: 1px solid rgba(255, 255, 255, 0.12);
}

.ln-hero-btn-secondary:hover {
  border-color: rgba(255, 255, 255, 0.25);
  background: rgba(255, 255, 255, 0.04);
}

.ln-hero-right {
  width: 100%;
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
}

.ln-hero-code-window {
  width: 100%;
  min-width: 0;
  max-width: 520px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(18, 15, 23, 0.45);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  box-shadow: 0 4px 32px rgba(0, 0, 0, 0.25), inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.ln-hero-code-titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 9px 9px 9px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.ln-hero-code-titlebar-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ln-hero-code-dots {
  display: flex;
  gap: 6px;
}
`;
