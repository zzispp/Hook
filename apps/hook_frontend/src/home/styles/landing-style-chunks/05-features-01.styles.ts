export const landingStyles05Features01 = String.raw`
/* ══════════════════════════════════════════════
   Features Section
   ══════════════════════════════════════════════ */

.ln-features-section {
  position: relative;
  width: 100%;
  padding: 80px 0 100px;
  background: #120F17;
  z-index: 4;
}

.ln-features-inner {
  max-width: 1324px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ─── Title ─── */

.ln-features-title {
  font-family: 'Geist Pixel Line', sans-serif;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 500;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: #fff;
  margin: 0 0 40px;
}

/* ─── Grid ─── */

.ln-features-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 16px;
}

.ln-features-card--span-5 { grid-column: span 5; }
.ln-features-card--span-4 { grid-column: span 4; }
.ln-features-card--span-3 { grid-column: span 3; }

/* ─── Card ─── */

.ln-features-card {
  background: rgba(18, 15, 23, 0.45);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  box-shadow:
    0 4px 32px rgba(0, 0, 0, 0.25),
    inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
  transition: border-color 0.3s ease, translate 0.3s ease;
  overflow: hidden;
}

.ln-features-card:hover {
  border-color: rgba(255, 255, 255, 0.15);
  translate: 0 -2px;
}

/* ─── Card Visual ─── */

.ln-features-card-visual {
  height: 180px;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

/* ─── Card Body ─── */

.ln-features-card-body {
  padding: 18px 22px 22px;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}

.ln-features-card-body h3 {
  font-family: 'Geist', sans-serif;
  font-size: 15px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 6px;
  letter-spacing: -0.01em;
}

.ln-features-card-body p {
  font-family: 'Geist', sans-serif;
  font-size: 13px;
  line-height: 1.55;
  color: rgba(255, 255, 255, 0.5);
  margin: 0;
}

/* ═══════════════════════════════════════
   1. Component Marquee
   ═══════════════════════════════════════ */

.ln-feat-marquee {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 8px;
  padding: 0 0;
  mask-image: linear-gradient(90deg, transparent 0%, #000 15%, #000 85%, transparent 100%);
  -webkit-mask-image: linear-gradient(90deg, transparent 0%, #000 15%, #000 85%, transparent 100%);
}

.ln-feat-marquee-track {
  overflow: hidden;
  width: 100%;
}

.ln-feat-marquee-scroll {
  display: flex;
  gap: 8px;
  width: max-content;
  animation: marqueeScroll 25s linear infinite;
}

.ln-feat-marquee-scroll--rev {
  animation: marqueeScrollRev 28s linear infinite;
}

@keyframes marqueeScroll {
  0% { transform: translateX(0); }
  100% { transform: translateX(-50%); }
}

@keyframes marqueeScrollRev {
  0% { transform: translateX(-50%); }
  100% { transform: translateX(0); }
}

.ln-feat-pill {
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  text-decoration: none;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 7px;
  padding: 5px 12px;
  white-space: nowrap;
  flex-shrink: 0;
  transition: border-color 0.2s, color 0.2s;
}

.ln-feat-pill:hover {
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.85);
}

/* ═══════════════════════════════════════
   2. Free Tools (Floating Boxes)
   ═══════════════════════════════════════ */

.ln-feat-tools {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ln-feat-tool-box {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  background: #1a1520;
  border: 1px solid rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.5);
  z-index: 2;
}

.ln-feat-tool-box--center {
  width: 56px;
  height: 56px;
  top: calc(50% - 40px);
  left: calc(50% - 28px);
  color: rgba(255, 255, 255, 0.6);
}

.ln-feat-tool-box--left {
  width: 46px;
  height: 46px;
  top: calc(50% + 6px);
  left: calc(50% - 58px);
  transform: rotate(-12deg);
}

.ln-feat-tool-box--right {
  width: 46px;
  height: 46px;
  top: calc(50% + 6px);
  left: calc(50% + 12px);
  transform: rotate(12deg);
}

/* ═══════════════════════════════════════
   3. Category Orbits
   ═══════════════════════════════════════ */

.ln-feat-orbit {
  position: relative;
  width: 280px;
  height: 280px;
}

`;
