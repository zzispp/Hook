export const landingStyles07LiveDemo02 = String.raw`
  background-color: #fff;
  transition: .3s;
  border-radius: 50%;
}

input:checked + .ln-slider {
  background-color: var(--accent-theme);
}

input:checked + .ln-slider:before {
  transform: translateX(18px);
}

.ln-send-btn {
  background: var(--accent-theme);
  color: var(--accent-theme-contrast, #fff);
  border: none;
  border-radius: 8px;
  padding: 12px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-family: inherit;
}

.ln-send-btn:hover:not(:disabled) {
  background: var(--accent-theme-hover);
  box-shadow: var(--shadow-primary);
}

.ln-send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ─── Terminal Panel ─── */

.ln-playground-terminal {
  background: #09070c;
  border: 1px solid var(--border-primary);
  border-radius: 14px;
  box-shadow: var(--shadow-card);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
  min-height: 480px;
}

.ln-terminal-head {
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.ln-terminal-dots {
  display: flex;
  gap: 6px;
}

.ln-terminal-dots span {
  width: 9px;
  height: 9px;
  border-radius: 50%;
}

.ln-terminal-dots span:nth-child(1) { background: #ff5f56; }
.ln-terminal-dots span:nth-child(2) { background: #ffbd2e; }
.ln-terminal-dots span:nth-child(3) { background: #27c93f; }

.ln-terminal-title {
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
}

.ln-terminal-body {
  padding: 16px;
  font-family: 'Geist Mono', monospace;
  font-size: 12.5px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.85);
  flex-grow: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ln-trace-log {
  margin: 0;
  display: flex;
  gap: 8px;
}

.ln-trace-time {
  color: rgba(255, 255, 255, 0.3);
  user-select: none;
  flex-shrink: 0;
}

.ln-trace-text {
  word-break: break-all;
}

.ln-trace-tag {
  display: inline-block;
  padding: 1px 4px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 500;
  margin-right: 6px;
  line-height: 1.2;
}

.ln-trace-tag--client { background: rgba(56, 189, 248, 0.12); color: #38bdf8; }
.ln-trace-tag--gateway { background: rgba(168, 85, 247, 0.12); color: #a855f7; }
.ln-trace-tag--upstream { background: rgba(234, 179, 8, 0.12); color: #eab308; }
.ln-trace-tag--error { background: rgba(239, 68, 68, 0.12); color: #ef4444; }
.ln-trace-tag--success { background: rgba(16, 185, 129, 0.12); color: #10b981; }

.ln-stream-output {
  margin-top: 8px;
  border-top: 1px dashed rgba(255, 255, 255, 0.08);
  padding-top: 12px;
}

.ln-stream-title {
  font-size: 10.5px;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  margin-bottom: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.ln-stream-text-box {
  background: rgba(255, 255, 255, 0.015);
  border: 1px solid rgba(255, 255, 255, 0.04);
  border-radius: 6px;
  padding: 12px;
  min-height: 110px;
  white-space: pre-wrap;
  word-break: break-all;
  color: rgba(255, 255, 255, 0.9);
}

.ln-stream-think {
  color: rgba(255, 255, 255, 0.45);
  border-left: 2px solid rgba(255, 255, 255, 0.12);
  padding-left: 10px;
  margin-bottom: 8px;
  font-style: italic;
}

.ln-stream-cursor {
  display: inline-block;
  width: 6px;
  height: 13px;
  background: var(--accent-theme);
  margin-left: 2px;
  animation: ln-blink 1s step-start infinite;
  vertical-align: middle;
}

@keyframes ln-blink {
  50% { opacity: 0; }
}

@media (max-width: 900px) {
  .ln-playground {
    grid-template-columns: 1fr;
  }
  .ln-playground-terminal {
    min-height: 400px;
  }
}
`;
