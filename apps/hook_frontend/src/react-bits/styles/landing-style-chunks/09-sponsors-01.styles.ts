export const landingStyles09Sponsors01 = String.raw`
/* ══════════════════════════════════════════════
   Supported Channels / Providers Grid (Landing)
   ══════════════════════════════════════════════ */

.ln-sp-section {
  position: relative;
  width: 100%;
  padding: 80px 0 100px;
  background: var(--bg-section-alt);
  border-top: 1px solid var(--border-primary);
  z-index: 4;
}

.ln-sp-inner {
  max-width: 1324px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ─── Title ─── */

.ln-sp-title {
  font-family: 'Geist', sans-serif;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin: 0 0 12px;
}

.ln-sp-subtitle {
  font-size: 15px;
  color: var(--text-muted);
  margin-bottom: 40px;
  font-family: 'Geist', sans-serif;
}

/* ─── Grid ─── */

.ln-providers-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

/* ─── Card ─── */

.ln-prov-card {
  display: flex;
  align-items: center;
  gap: 16px;
  border-radius: 14px;
  border: 1px solid var(--border-primary);
  background: var(--bg-card);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  padding: 18px 16px;
  text-decoration: none;
  transition: all 0.25s cubic-bezier(0.21, 0.47, 0.32, 0.98);
  min-width: 0;
  box-shadow: var(--shadow-card-soft);
}

.ln-prov-card:hover {
  border-color: var(--palette-primary-main);
  background: var(--bg-hover);
  transform: translateY(-2px);
  box-shadow: var(--shadow-card);
}

.ln-prov-icon-wrap {
  width: 46px;
  height: 46px;
  border-radius: 10px;
  background: var(--bg-input);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 1px solid var(--border-secondary);
}

.ln-prov-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex-grow: 1;
}

.ln-prov-name-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.ln-prov-name {
  font-family: 'Geist', sans-serif;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.ln-prov-desc {
  font-family: 'Geist', sans-serif;
  font-size: 11.5px;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ln-prov-tag {
  font-size: 9.5px;
  font-family: 'Geist Mono', monospace;
  color: var(--palette-primary-main);
  background: var(--bg-code-token);
  border: 1px solid var(--border-subtle);
  padding: 1px 4px;
  border-radius: 4px;
  line-height: 1.2;
}

/* ─── Responsive ─── */

@media (max-width: 1100px) {
  .ln-providers-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 900px) {
  .ln-providers-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 640px) {
  .ln-providers-grid {
    grid-template-columns: 1fr;
  }
}
`;
