export const landingStyles03Navbar02 = String.raw`

  .ln-navbar-links {
    display: none;
  }

  .ln-navbar-divider {
    display: none;
  }

  .ln-navbar-browse {
    display: none;
  }

  .ln-navbar-pro {
    height: 32px;
    padding: 0 12px;
    font-size: 12px;
    border-width: 3px;
    border-radius: 8px;
  }

  .ln-navbar-github {
    display: none;
  }
}

/* Hamburger button — hidden on desktop */
.ln-navbar-hamburger {
  display: none;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  width: 36px;
  height: 36px;
  padding: 9px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  background: rgba(18, 15, 23, 0.45);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  cursor: pointer;
  transition: background 0.2s ease, border-color 0.2s ease;
}

.ln-navbar-hamburger:hover {
  border-color: rgba(255, 255, 255, 0.15);
  background: rgba(18, 15, 23, 0.55);
}

.ln-navbar-hamburger span {
  display: block;
  width: 100%;
  height: 1.5px;
  background: #fff;
  border-radius: 1px;
  transition: transform 0.25s ease, opacity 0.25s ease;
}

.ln-navbar-hamburger.open span:nth-child(1) {
  transform: translateY(5.5px) rotate(45deg);
}

.ln-navbar-hamburger.open span:nth-child(2) {
  opacity: 0;
}

.ln-navbar-hamburger.open span:nth-child(3) {
  transform: translateY(-5.5px) rotate(-45deg);
}

/* Mobile dropdown menu */
.ln-navbar-mobile-menu {
  display: none;
  flex-direction: column;
  gap: 2px;
  position: absolute;
  top: calc(100% + 8px);
  right: 5px;
  min-width: 200px;
  padding: 8px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(18, 15, 23, 0.85);
  backdrop-filter: blur(32px) saturate(1.3);
  -webkit-backdrop-filter: blur(32px) saturate(1.3);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  pointer-events: auto;
  animation: ln-menu-slide-in 0.2s ease;
  z-index: 1001;
}

@keyframes ln-menu-slide-in {
  from {
    opacity: 0;
    transform: translateY(-6px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.ln-navbar-mobile-link {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-family: 'Geist Mono', monospace;
  font-size: 13px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: rgba(255, 255, 255, 0.7);
  padding: 12px 14px;
  border-radius: 10px;
  transition: background 0.15s ease, color 0.15s ease;
}

.ln-navbar-mobile-link::after {
  content: '→';
  font-size: 16px;
  color: inherit;
  opacity: 0;
  transform: translateX(-6px);
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.ln-navbar-mobile-link:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #fff;
}

.ln-navbar-mobile-link:hover::after {
  opacity: 1;
  transform: translateX(0);
}

.ln-navbar-mobile-pro {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px;
  margin-bottom: 6px;
  border-radius: 12px;
  background: linear-gradient(125deg, rgba(var(--pro-glow-dark, 124, 58, 237), 0.35), rgba(var(--pro-glow, 168, 85, 247), 0.35), rgba(var(--pro-glow-light, 217, 70, 239), 0.3));
  border: 1px solid rgba(var(--pro-glow, 168, 85, 247), 0.35);
  box-shadow: 0 0 24px rgba(var(--pro-glow, 168, 85, 247), 0.22);
  text-decoration: none;
  color: #fff;
  transition: transform 0.15s ease, box-shadow 0.2s ease;
}

.ln-navbar-mobile-pro:active {
  transform: scale(0.985);
}

.ln-navbar-mobile-pro-title {
  flex: 1;
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.2px;
}

.ln-navbar-mobile-pro-arrow {
  font-size: 18px;
  font-weight: 600;
  color: #fff;
}

@media (max-width: 768px) {
  .ln-navbar-hamburger {
    display: flex;
  }

  .ln-navbar-mobile-menu {
    display: flex;
  }

  .ln-navbar-inner {
    width: 100%;
    max-width: 100%;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 5px 0 5px;
    border-radius: 16px;
    border: 1px solid transparent;
    background: transparent;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
    transition: background 0.5s ease, border-color 0.5s ease, padding 0.5s ease;
    pointer-events: auto;
    position: relative;
  }

  .ln-navbar.ln-navbar-scrolled .ln-navbar-inner {
    max-width: 100%;
    padding: 0 5px 0 16px;
    background: rgba(18, 15, 23, 0.45);
    backdrop-filter: blur(24px) saturate(1.4);
    -webkit-backdrop-filter: blur(24px) saturate(1.4);
    border-color: rgba(255, 255, 255, 0.04);
  }
}
`;
