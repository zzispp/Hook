export const landingStyles02Landing01 = String.raw`
.landing-wrapper {
  min-height: 100dvh;
  position: relative;
  overflow-x: hidden;
}

.landing-wrapper::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 300px;
  height: 100vh;
  background: linear-gradient(to right, var(--bg-body), transparent);
  z-index: 2;
  pointer-events: none;
}

.landing-wrapper::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  width: 300px;
  height: 100vh;
  background: linear-gradient(to left, var(--bg-body), transparent);
  z-index: 2;
  pointer-events: none;
}

.landing-wrapper.no-side-fades::before,
.landing-wrapper.no-side-fades::after {
  display: none;
}

.landing-content {
  position: relative;
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  min-height: 1000px;
  max-width: calc(1200px + 6em);
  margin: 0 auto;
  padding: 0 4em;
  z-index: 10;
}

/* Landing content responsive min-height */
@media (max-width: 1024px) {
  .landing-content {
    min-height: 700px;
  }
}

@media (max-width: 768px) {
  .landing-content {
    min-height: 600px;
  }
}

@media (max-width: 480px) {
  .landing-content {
    min-height: 500px;
  }
}

.landing-title {
  user-select: none;
  text-align: center;
  font-size: 4rem;
  font-weight: 600;
  position: relative;
  z-index: 6;
  color: var(--text-primary);
  margin-bottom: 1rem;
  max-width: 22ch;
  letter-spacing: -3px;
  line-height: 1;
  /* Large text-shadow removed for scroll performance */
}

@media (max-width: 768px) {
  .landing-title {
    white-space: normal;
  }
}

.landing-subtitle {
  user-select: none;
  text-align: center;
  overflow: visible !important;
  font-size: 1.2rem;
  font-weight: 300;
  line-height: 1.6;
  color: #d3d3d3b7;
  /* Large text-shadow removed for scroll performance */
  font-weight: 400;
  max-width: 40ch;
  margin-bottom: 2rem;
  z-index: 6;
}

.landing-gradient-blur {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  aspect-ratio: 1;
  height: 1200px;
  background: radial-gradient(circle, rgba(6, 0, 16, 0.6) 0%, transparent 70%);
  pointer-events: none;
}

.hero-new-badge-container {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  border: 1px solid var(--border-subtle);
  background: rgba(6, 0, 16, 0.8);
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.15);
  /* backdrop-filter removed for performance */
  padding: 0.6em 0.8em 0.6em 0.6em;
  border-radius: 50px;
  margin-bottom: 1.5rem;
  color: #ccc;
  z-index: 6;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.hero-tag-fade {
  position: relative;
  z-index: 8;
}

.hero-new-badge {
  font-size: 14px;
  font-weight: 700;
  letter-spacing: -0.5px;
  background: var(--color-primary);
  padding: 0.4em 0.8em 0.4em 1.2em;
  border-radius: 50px;
  display: flex;
  align-items: center;
  gap: 0.6em;
}

.hero-new-badge-text {
  display: flex;
  align-items: center;
  gap: 0.4em;
}

.hero-new-badge-container:hover {
  box-shadow:
    0 0 60px rgba(124, 58, 237, 0.2),
    0 0 120px rgba(139, 92, 246, 0.2),
    0 0 180px rgba(109, 40, 217, 0.2),
    0 12px 40px rgba(0, 0, 0, 0.4);
  transform: scale(1.01);
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  animation-duration: 1.5s;
}

.hero-split {
  white-space: nowrap !important;
  overflow: visible !important;
}

.landing-button {
  position: relative;
  background: var(--color-primary);
  background-size: 200% 200%;
  font-weight: 500;
  font-size: 1rem;
  border-radius: var(--radius-full);
  border: none;
  letter-spacing: -0.2px;
  height: 55px;
  padding: 0 2em 0 2em;
  color: white;
  cursor: pointer;
  isolation: isolate;
  z-index: 15;
  box-shadow:
    0 0 40px rgba(124, 58, 237, 0.2),
    0 0 80px rgba(139, 92, 246, 0.1),
    0 8px 32px rgba(0, 0, 0, 0.1);
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  /* glow-pulse animation removed for performance - filter animations are expensive */
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.button-arrow-circle {
  display: none;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  background: rgba(255, 255, 255, 0.9);
  border-radius: 50%;
  color: #4c1d95;
  transition: all 0.3s ease;
  /* backdrop-filter removed for performance */
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  flex-shrink: 0;
}

.landing-button:hover .button-arrow-circle {
  background: rgba(255, 255, 255, 1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.landing-button span {
  position: relative;
`;
