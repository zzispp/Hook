export const landingStyles07LiveDemo01 = String.raw`
/* ══════════════════════════════════════════════
   Live Demo / API Gateway Playground
   ══════════════════════════════════════════════ */

:root {
  --accent-theme: var(--color-primary);
  --accent-theme-hover: var(--color-primary-hover);
  --accent-theme-bg: var(--bg-code-token);
  --accent-theme-border: var(--border-subtle);
}

[data-color-scheme='dark'] {
  --accent-theme: #38f2b2;
  --accent-theme-hover: #2ce0a0;
  --accent-theme-bg: rgba(56, 242, 178, 0.08);
  --accent-theme-border: rgba(56, 242, 178, 0.25);
}

.ln-demo-section {
  position: relative;
  width: 100%;
  padding: 80px 0 100px;
  background: var(--bg-body);
  border-top: 1px solid var(--border-primary);
  z-index: 4;
}

.ln-demo-inner {
  max-width: 1324px;
  margin: 0 auto;
  padding: 0 24px;
}

/* ─── Title ─── */

.ln-demo-title {
  font-family: 'Geist', sans-serif;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 600;
  line-height: 1.1;
  letter-spacing: -0.02em;
  color: var(--text-primary);
  margin: 0 0 10px;
  text-align: center;
}

.ln-demo-subtitle {
  font-size: 16px;
  color: var(--text-muted);
  margin-bottom: 40px;
  text-align: center;
  font-family: 'Geist', sans-serif;
}

/* ─── Playground Layout ─── */

.ln-playground {
  display: grid;
  grid-template-columns: 1fr 1.2fr;
  gap: 24px;
  align-items: stretch;
}

/* ─── Control Panel ─── */

.ln-playground-control {
  background: var(--bg-card);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  border: 1px solid var(--border-primary);
  border-radius: 14px;
  padding: 24px;
  box-shadow: var(--shadow-card-soft);
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.ln-control-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ln-control-label {
  font-family: 'Geist', sans-serif;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
}

.ln-select {
  background: var(--bg-input);
  border: 1px solid var(--border-secondary);
  color: var(--text-primary);
  border-radius: 8px;
  padding: 10px 12px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
  width: 100%;
  font-family: inherit;
}

.ln-select option {
  background: var(--bg-elevated);
  color: var(--text-primary);
}

.ln-select:focus {
  border-color: var(--accent-theme);
}

.ln-prompts-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ln-prompt-btn {
  background: var(--bg-input);
  border: 1px solid var(--border-secondary);
  color: var(--text-secondary);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: all 0.2s;
  font-family: inherit;
}

.ln-prompt-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-theme-border);
  color: var(--text-primary);
}

.ln-prompt-btn.active {
  background: var(--accent-theme-bg);
  border-color: var(--accent-theme);
  color: var(--accent-theme);
}

.ln-textarea {
  background: var(--bg-input);
  border: 1px solid var(--border-secondary);
  color: var(--text-primary);
  border-radius: 8px;
  padding: 12px;
  font-size: 14px;
  min-height: 70px;
  resize: vertical;
  outline: none;
  font-family: inherit;
  line-height: 1.4;
}

.ln-textarea:focus {
  border-color: var(--accent-theme);
}

.ln-toggle-wrapper {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-input);
  border: 1px solid var(--border-secondary);
  border-radius: 8px;
  padding: 12px;
}

.ln-toggle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ln-toggle-title {
  font-family: 'Geist', sans-serif;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.ln-toggle-desc {
  font-family: 'Geist', sans-serif;
  font-size: 11px;
  color: var(--text-muted);
}

.ln-switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.ln-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.ln-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border-strong);
  transition: .3s;
  border-radius: 22px;
}

.ln-slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
`;
