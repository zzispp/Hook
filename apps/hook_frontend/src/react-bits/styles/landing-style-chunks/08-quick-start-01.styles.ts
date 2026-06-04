export const landingStyles08QuickStart01 = String.raw`
/* ══════════════════════════════════════════════
   QuickStart
   ══════════════════════════════════════════════ */

.ln-qs-section {
  position: relative;
  width: 100%;
  padding: 80px 0 200px 0;
  background: #0e0b13;
  z-index: 4;
}

.ln-qs-inner {
  max-width: 860px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ─── Header ─── */

.ln-qs-header {
  text-align: center;
  margin-bottom: 40px;
}

.ln-qs-title {
  font-family: 'Geist Pixel Line', sans-serif;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 500;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: #fff;
  margin: 0 0 14px;
}

/* ═══════════════════════════════════════
   Terminal Window
   ═══════════════════════════════════════ */

.ln-qs-terminal-wrap {
  position: relative;
}

.ln-qs-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 70%;
  height: 120%;
  background: radial-gradient(ellipse, rgba(255, 255, 255, 0.02) 0%, transparent 70%);
  pointer-events: none;
  z-index: 0;
}

.ln-qs-terminal {
  position: relative;
  z-index: 1;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(18, 15, 23, 0.6);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  box-shadow:
    0 4px 32px rgba(0, 0, 0, 0.3),
    inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
  overflow: visible;
}

/* ── Tab bar ── */

.ln-qs-tab-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.ln-qs-tabs {
  display: flex;
}

.ln-qs-tab {
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
  background: transparent;
  border: none;
  border-bottom: 1px solid transparent;
  padding: 12px 16px;
  cursor: pointer;
  transition: color 0.2s, border-color 0.2s;
}

.ln-qs-tab:hover {
  color: rgba(255, 255, 255, 0.6);
}

.ln-qs-tab--active {
  color: rgba(255, 255, 255, 0.85);
  border-bottom-color: rgba(255, 255, 255, 0.5);
}

/* ── Command area ── */

.ln-qs-cmd-area {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 24px 20px;
}

.ln-qs-cmd-line {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
  overflow-x: auto;
}

.ln-qs-cmd-line::-webkit-scrollbar {
  height: 0;
}

.ln-qs-prompt {
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.25);
  flex-shrink: 0;
}

.ln-qs-cmd-text {
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  white-space: nowrap;
  line-height: 1.5;
}

/* ── Copy button ── */

.ln-qs-copy {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 7px;
  border: 1px solid rgba(255, 255, 255, 0.12) !important;
  background: rgba(255, 255, 255, 0.03) !important;
  color: rgba(255, 255, 255, 0.4) !important;
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.2s, border-color 0.2s, background 0.2s;
}

.ln-qs-copy:hover {
  color: #ffffff !important;
  border-color: rgba(255, 255, 255, 0.3) !important;
  background: rgba(255, 255, 255, 0.08) !important;
}

.ln-qs-copy--done {
  color: #38f2b2 !important;
  border-color: rgba(56, 242, 178, 0.3) !important;
  background: rgba(56, 242, 178, 0.08) !important;
}
/* ── Hint ── */

.ln-qs-hint {
  font-family: 'Geist', sans-serif;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.25);
  text-align: center;
  margin: 18px 0 0;
}

/* ═══════════════════════════════════════
   Responsive
   ═══════════════════════════════════════ */

@media (max-width: 480px) {
  .ln-qs-section {
    padding: 60px 0 80px;
  }

  .ln-qs-inner {
    padding: 0 16px;
  }

  .ln-qs-cmd-area {
    padding: 20px 14px;
  }

  .ln-qs-cmd-text {
    font-size: 11px;
  }
}
`;
