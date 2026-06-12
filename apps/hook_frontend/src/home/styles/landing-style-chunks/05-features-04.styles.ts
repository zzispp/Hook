export const landingStyles05Features04 = String.raw`

/* ═══════════════════════════════════════
   Responsive
   ═══════════════════════════════════════ */

@media (max-width: 1024px) {
  .ln-features-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .ln-features-card--span-5,
  .ln-features-card--span-4,
  .ln-features-card--span-3 {
    grid-column: span 1;
  }
}

@media (max-width: 640px) {
  .ln-features-grid {
    grid-template-columns: 1fr;
  }

  .ln-features-section {
    padding: 60px 0 80px;
  }

  .ln-features-inner {
    padding: 0 16px;
  }

  .ln-features-title {
    font-size: 22px;
  }
}
`;
