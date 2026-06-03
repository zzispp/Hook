import { gsap } from 'gsap';
import { useRef, useEffect } from 'react';

export const Logo = () => {
  const svgRef = useRef<SVGSVGElement>(null);
  const iconRef = useRef<SVGGElement>(null);

  useEffect(() => {
    const svgEl = svgRef.current;
    const iconEl = iconRef.current;
    if (!svgEl || !iconEl) return undefined;

    gsap.set(iconEl, { transformOrigin: 'center center' });

    const handleEnter = () => gsap.to(iconEl, { rotation: 20, scale: 0.95, duration: 1, ease: 'elastic.out(1, 0.3)' });
    const handleLeave = () => gsap.to(iconEl, { rotation: 0, scale: 1, duration: 1, ease: 'elastic.out(2, 1)' });

    svgEl.addEventListener('mouseenter', handleEnter);
    svgEl.addEventListener('mouseleave', handleLeave);

    return () => {
      svgEl.removeEventListener('mouseenter', handleEnter);
      svgEl.removeEventListener('mouseleave', handleLeave);
    };
  }, []);

  return (
    <svg
      ref={svgRef}
      width="100"
      height="36"
      viewBox="0 0 100 36"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      style={{ color: 'var(--landing-logo-color)', cursor: 'pointer' }}
    >
      <g ref={iconRef}>
        <g stroke="currentColor" strokeWidth="5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M 28 4 V 20 A 12 12 0 0 1 4 20 A 6 6 0 0 1 10 14" />
        </g>
        <circle cx="19" cy="11" r="3.5" fill="currentColor" opacity="0.6" />
      </g>
      <text x="38" y="26" fontFamily="system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" fontWeight="800" fontSize="24" fill="currentColor" letterSpacing="-0.03em">Hook</text>
    </svg>
  );
};
