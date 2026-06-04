export const landingStyles02Landing02 = String.raw`
  z-index: 2;
  display: inline-block;
  transition: all 0.3s ease;
}

.landing-button:hover span {
  transform: scale(1.01);
}

.landing-button::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.4), transparent);
  transition: left 0.6s ease;
  z-index: 1;
}

.landing-button::after {
  content: '';
  position: absolute;
  inset: -2px;
  background: linear-gradient(
    45deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
  background-size: 200% 200%;
  border-radius: 50px;
  z-index: -1;
  animation: border-dance 4s linear infinite;
  opacity: 0;
  transition: opacity 0.3s ease;
}

.landing-button:hover {
  box-shadow:
    0 0 60px rgba(124, 58, 237, 0.2),
    0 0 120px rgba(139, 92, 246, 0.2),
    0 0 180px rgba(109, 40, 217, 0.2),
    0 12px 40px rgba(0, 0, 0, 0.4),
    inset 0 2px 0 rgba(255, 255, 255, 0.4),
    inset 0 -2px 0 rgba(0, 0, 0, 0.3);
  transform: scale(1.01);
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  animation-duration: 1.5s;
}

.landing-button:hover::before {
  left: 100%;
}

.landing-button:hover::after {
  opacity: 1;
}

.landing-button:active {
  transform: translateY(-2px) scale(1.02);
  transition: all 0.1s ease;
}

@keyframes glow-pulse {
  0% {
    filter: brightness(1) saturate(1);
  }

  100% {
    filter: brightness(1.1) saturate(1.2);
  }
}

@keyframes border-dance {
  0% {
    background-position: 0% 50%;
  }

  50% {
    background-position: 100% 50%;
  }

  100% {
    background-position: 0% 50%;
  }
}

@keyframes fadeInUp {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Hero text animations - CSS-based for performance */
.hero-text-animate {
  display: inline-block;
  opacity: 0;
  animation: heroTextFadeIn 0.8s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.hero-text-animate-delay {
  animation-delay: 0.15s;
}

.hero-subtitle-delay {
  animation-delay: 0.3s;
}

.hero-button-delay {
  animation-delay: 0.45s;
}

@keyframes heroTextFadeIn {
  0% {
    opacity: 0;
    transform: translateY(20px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes fadeInUpRotate1 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(-13deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(-13deg);
  }
}

@keyframes fadeInUpRotate2 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(10deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(10deg);
  }
}

@keyframes fadeInUpRotate3 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(-5deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(-5deg);
  }
}

@keyframes fadeInUpMobile {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes fadeInUpMobileRotate1 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(-13deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(-13deg);
  }
}

@keyframes fadeInUpMobileRotate2 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(10deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(10deg);
  }
}

@keyframes fadeInUpMobileRotate3 {
  0% {
    opacity: 0;
    transform: translateY(30px) scale(0.9) rotate(-5deg);
  }

  100% {
    opacity: 1;
    transform: translateY(0) scale(1) rotate(-5deg);
  }
}

.hero-main-content {
  flex: 1;
  display: flex;
`;
