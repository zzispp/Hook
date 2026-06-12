export const landingStyles02Landing03 = String.raw`
  flex-direction: column;
  align-items: center;
  max-width: 100%;
}

@media (max-width: 1440px) {
  .landing-title {
    font-size: 3.6rem;
  }

  .landing-subtitle {
    font-size: clamp(0.85rem, 1.2vw, 1.3rem);
  }
}

@media (max-width: 1024px) {
  .landing-content {
    margin-top: 250px;
    height: auto;
    padding: 0 2em;
    max-width: calc(1200px + 2em);
    flex-direction: column;
    align-items: center;
  }

  .hero-main-content {
    max-width: 100%;
    align-items: center;
    text-align: center;
    margin-bottom: 2rem;
  }

  .landing-subtitle {
    text-align: center !important;
  }

  .landing-wrapper > div[style*='position: absolute'][style*='width: 100vw'][style*='height: 100vh'] {
    opacity: 0.7;
  }
}

@media (max-width: 900px) {
  .landing-wrapper::before {
    width: 100px;
  }

  .landing-wrapper::after {
    width: 100px;
  }
}

@media (max-width: 768px) {
  .landing-title {
    font-size: clamp(1.6rem, 5vw, 3.2rem);
    text-align: center;
    max-width: 100%;
  }

  .landing-subtitle {
    font-size: clamp(0.75rem, 2vw, 1rem);
    text-align: center !important;
    max-width: 80%;
  }

  .landing-content {
    margin-top: 260px;
    align-items: center;
    padding: 0 1.5rem;
  }

  .landing-button {
    padding: 0 2em;
    font-size: 1rem;
    gap: 1.2rem;
  }

  .button-arrow-circle {
    width: 32px;
    height: 32px;
  }

  .landing-wrapper > div[style*='position: absolute'][style*='width: 100vw'][style*='height: 100vh'] {
    opacity: 0.5;
  }
}

@media (max-width: 480px) {
  .landing-title {
    font-size: clamp(1.6rem, 5vw, 1.8rem);
    letter-spacing: -1px;
  }

  .landing-content {
    margin-top: 210px;
  }

  .landing-button {
    font-size: 0.95rem;
    gap: 1rem;
  }

  .button-arrow-circle {
    width: 28px;
    height: 28px;
  }

  .landing-wrapper > div[style*='position: absolute'][style*='width: 100vw'][style*='height: 100vh'] {
    opacity: 0.3;
  }
}

@media (max-width: 375px) {
  .landing-content {
    margin-top: 150px;
  }

  .landing-button {
    font-size: 0.9rem;
    gap: 0.8rem;
  }

  .button-arrow-circle {
    width: 26px;
    height: 26px;
  }
}

/* Mobile hero background styles */
.mobile-hero-background-container {
  display: none;
  position: absolute;
  inset: 0;
  overflow: hidden;
  width: 100vw;
  height: 100vh;
  z-index: -1;
}

@media (max-width: 768px) {
  .mobile-hero-background-container {
    display: block;
  }
}

.mobile-hero-background-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0.6;
}
`;
