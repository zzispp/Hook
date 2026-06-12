export const landingStyles04Hero02 = String.raw`

.ln-hero-code-dots span {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
}

.ln-hero-code-dropdown {
  position: relative;
}

.ln-hero-code-dropdown-trigger {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  background: rgba(255, 255, 255, 0.01);
  border: 1px solid rgba(255, 255, 255, 0.06);
  padding: 0 10px;
  height: 30px;
  border-radius: 7px 7px 2px 7px;
  cursor: pointer;
  box-sizing: border-box;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.ln-hero-code-dropdown-trigger:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.1);
}

.ln-hero-code-caret {
  transition: transform 0.25s ease;
}

.ln-hero-code-caret.open {
  transform: rotate(180deg);
}

.ln-hero-code-dropdown-menu {
  position: absolute;
  top: calc(100% + 9px);
  right: -10px;
  min-width: 140px;
  padding: 4px;
  border-radius: 0 0 0 7px;
  background: rgba(18, 15, 23, 0.95);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 10;
  opacity: 0;
  transform: translateY(-4px) scale(0.97);
  pointer-events: none;
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.ln-hero-code-dropdown-menu.open {
  opacity: 1;
  transform: translateY(0) scale(1);
  pointer-events: auto;
}

.ln-hero-code-dropdown-item {
  display: block;
  width: 100%;
  text-align: left;
  font-family: 'Geist Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  background: transparent;
  border: none;
  padding: 7px 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.ln-hero-code-dropdown-item:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.8);
}

.ln-hero-code-dropdown-item.active {
  color: rgba(255, 255, 255, 0.8);
}

.ln-hero-code-body {
  padding: 4px 8px;
  overflow: hidden;
}

.ln-hero-code-pre {
  margin: 0;
  padding: 12px 8px;
  background: transparent;
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  line-height: 1.7;
  color: #988BC7;
  white-space: pre;
  overflow-x: auto;
  overflow-y: hidden;
}

.ln-hero-code-hint {
  font-family: 'Geist Mono', monospace;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.2);
  padding: 0 16px 10px;
  margin: 0;
  text-align: right;
}

/* ── Responsive ── */

@media (max-width: 1275px) {
  .ln-hero-content {
    grid-template-columns: 1fr;
    gap: 40px;
  }

  .ln-hero-right {
    justify-content: flex-start;
  }

  .ln-hero-code-window {
    max-width: 100%;
  }
}

@media (max-width: 1024px) {
  .ln-hero-content {
    padding: clamp(110px, 14vw, 160px) 24px 60px;
  }

  .ln-hero-right {
    justify-content: flex-start;
  }

  .ln-hero-headline {
    font-size: clamp(36px, 6vw, 48px);
  }

  .ln-hero-description {
    max-width: 45ch;
  }
}

@media (max-width: 640px) {
  .ln-hero-content {
    padding: clamp(90px, 16vw, 120px) 16px 48px;
  }

  .ln-hero-headline {
    font-size: clamp(30px, 8vw, 40px);
  }

  .ln-hero-description {
    font-size: 15px;
  }

  .ln-hero-btn {
    font-size: 13px;
    padding: 10px 20px;
  }

  .ln-hero-tag {
    font-size: 11px;
  }

  .ln-hero-tag-new {
    font-size: 11px;
    padding: 3px 8px;
  }
}
`;
