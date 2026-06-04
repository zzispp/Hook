export const landingStyles05Features03 = String.raw`
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  flex-shrink: 0;
}

.ln-feat-aichat-dots {
  display: flex;
  gap: 5px;
}

.ln-feat-aichat-dots span {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
}

.ln-feat-aichat-title {
  font-family: 'Geist', sans-serif;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.25);
  letter-spacing: 0.03em;
}

/* Prompt row */
.ln-feat-aichat-prompt-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  flex-shrink: 0;
}

.ln-feat-aichat-chevron {
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  flex-shrink: 0;
}

.ln-feat-aichat-prompt {
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
  white-space: nowrap;
  min-height: 1em;
}

.ln-feat-aichat-cursor {
  width: 2px;
  height: 13px;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 1px;
  flex-shrink: 0;
  animation: cursorBlink 0.8s ease-in-out infinite;
}

/* Thinking dots */
.ln-feat-aichat-thinking {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 12px 12px;
  flex-shrink: 0;
}

.ln-feat-aichat-thinking span {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.4);
  opacity: 0.4;
  animation: thinkPulse 1.2s ease-in-out infinite;
}

.ln-feat-aichat-thinking span:nth-child(2) { animation-delay: 0.2s; }
.ln-feat-aichat-thinking span:nth-child(3) { animation-delay: 0.4s; }

@keyframes thinkPulse {
  0%, 100% { opacity: 0.2; transform: scale(0.85); }
  50% { opacity: 0.7; transform: scale(1); }
}

/* Code block */
.ln-feat-aichat-code-block {
  display: flex;
  flex-direction: column;
  padding: 8px 0;
  flex: 1;
  min-height: 0;
}

.ln-feat-aichat-code-line {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 3px 12px;
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  line-height: 1.6;
  min-height: 22px;
  white-space: pre;
}

.ln-feat-aichat-ln {
  color: rgba(255, 255, 255, 0.12);
  width: 20px;
  flex-shrink: 0;
  text-align: right;
  margin-right: 12px;
  font-size: 10px;
}

/* Syntax tokens */
.ac-kw { color: rgba(255, 255, 255, 0.5); }
.ac-comp { color: rgba(255, 255, 255, 0.8); }
.ac-tag { color: rgba(255, 255, 255, 0.45); }
.ac-attr { color: rgba(255, 255, 255, 0.6); }
.ac-punc { color: rgba(255, 255, 255, 0.3); }
.ac-num { color: rgba(255, 255, 255, 0.7); }
.ac-str { color: rgba(255, 255, 255, 0.55); }

@keyframes cursorBlink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* ═══════════════════════════════════════
   6. Quota Stats Card
   ═══════════════════════════════════════ */

.ln-feat-quota {
  width: 100%;
  height: 100%;
  padding: 16px 18px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.ln-feat-quota-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
}

.ln-feat-quota-label {
  font-family: 'Geist', sans-serif;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  letter-spacing: 0.02em;
}

.ln-feat-quota-value {
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
  letter-spacing: -0.01em;
}

.ln-feat-quota-bar-track {
  width: 100%;
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.05);
  overflow: hidden;
}

.ln-feat-quota-bar-fill {
  height: 100%;
  border-radius: 3px;
  background: linear-gradient(90deg, rgba(139, 92, 246, 0.5) 0%, rgba(59, 130, 246, 0.5) 100%);
}

.ln-feat-quota-rows {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
  justify-content: center;
}

.ln-feat-quota-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 10px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.ln-feat-quota-row-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.ln-feat-quota-row-dot--a { background: rgba(139, 92, 246, 0.6); }
.ln-feat-quota-row-dot--b { background: rgba(59, 130, 246, 0.6); }
.ln-feat-quota-row-dot--c { background: rgba(16, 185, 129, 0.6); }

.ln-feat-quota-row-name {
  font-family: 'Geist', sans-serif;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.55);
  flex: 1;
}

.ln-feat-quota-row-val {
  font-family: 'Geist Mono', monospace;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.35);
  white-space: nowrap;
}
`;
