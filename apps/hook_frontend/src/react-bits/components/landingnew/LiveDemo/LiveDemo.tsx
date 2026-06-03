import type { ReactNode } from 'react';

import { motion } from 'motion/react';
import { lazy, Suspense, useState, useEffect } from 'react';
import { FiHome, FiStar, FiHeart, FiSearch, FiSettings } from 'react-icons/fi';

import { Link } from 'src/react-bits/router';

const ShapeGrid = lazy(() => import('../../../content/Backgrounds/ShapeGrid/ShapeGrid'));
const ShinyText = lazy(() => import('../../../content/TextAnimations/ShinyText/ShinyText'));
const MagicRings = lazy(() => import('../../../content/Animations/MagicRings/MagicRings'));
const Dock = lazy(() => import('../../../content/Components/Dock/Dock'));

const DEMO_COLOR_DEFAULTS = {
  shapeBorder: 'rgba(105, 80, 232, 0.18)',
  shapeHover: 'rgba(105, 80, 232, 0.08)',
  ringPrimary: '#6950E8',
  ringSecondary: '#FFAB00',
};

type DemoColors = typeof DEMO_COLOR_DEFAULTS;

type DemoCard = {
  readonly category: string;
  readonly component: string;
  readonly href: string;
  readonly span: number;
  readonly tall?: boolean;
  readonly render: (colors: DemoColors) => ReactNode;
};

function readCssVar(styles: CSSStyleDeclaration, name: string, fallback: string): string {
  const value = styles.getPropertyValue(name).trim();
  return value || fallback;
}

function readDemoColors(): DemoColors {
  if (typeof window === 'undefined') return DEMO_COLOR_DEFAULTS;

  const styles = window.getComputedStyle(document.documentElement);
  return {
    shapeBorder: readCssVar(styles, '--demo-shape-border', DEMO_COLOR_DEFAULTS.shapeBorder),
    shapeHover: readCssVar(styles, '--demo-shape-hover', DEMO_COLOR_DEFAULTS.shapeHover),
    ringPrimary: readCssVar(styles, '--demo-ring-primary', DEMO_COLOR_DEFAULTS.ringPrimary),
    ringSecondary: readCssVar(styles, '--demo-ring-secondary', DEMO_COLOR_DEFAULTS.ringSecondary),
  };
}

function useDemoColors(): DemoColors {
  const [colors, setColors] = useState(readDemoColors);

  useEffect(() => {
    const update = () => setColors(readDemoColors());
    const observer = new MutationObserver(update);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-color-scheme', 'data-react-bits-home'],
    });
    update();
    return () => observer.disconnect();
  }, []);

  return colors;
}

const CARDS: readonly DemoCard[] = [
  {
    category: 'Backgrounds',
    component: 'ShapeGrid',
    href: '/backgrounds/shape-grid',
    span: 7,
    tall: true,
    render: (colors) => (
      <Suspense fallback={null}>
        <ShapeGrid
          shape="hexagon"
          squareSize={48}
          borderColor={colors.shapeBorder}
          hoverFillColor={colors.shapeHover}
          direction="right"
          speed={0.3}
        />
      </Suspense>
    ),
  },
  {
    category: 'Animations',
    component: 'MagicRings',
    href: '/animations/magic-rings',
    span: 5,
    tall: true,
    render: (colors) => (
      <Suspense fallback={null}>
        <div className="ln-demo-rings-wrap">
          <MagicRings
            color={colors.ringPrimary}
            colorTwo={colors.ringSecondary}
            ringCount={5}
            speed={0.6}
            lineThickness={1}
            opacity={0.6}
          />
        </div>
      </Suspense>
    ),
  },
  {
    category: 'Text Animations',
    component: 'ShinyText',
    href: '/text-animations/shiny-text',
    span: 4,
    render: () => (
      <Suspense fallback={null}>
        <div className="ln-demo-center">
          <ShinyText
            text="Shiny Text"
            speed={2.5}
            className="ln-demo-shiny"
          />
        </div>
      </Suspense>
    ),
  },
  {
    category: 'Components',
    component: 'Dock',
    href: '/components/dock',
    span: 8,
    render: () => (
      <Suspense fallback={null}>
        <div className="ln-demo-dock-wrap">
          <Dock
            items={[
              { icon: <FiHome size={20} />, label: 'Home' },
              { icon: <FiSearch size={20} />, label: 'Search' },
              { icon: <FiStar size={20} />, label: 'Favorites' },
              { icon: <FiHeart size={20} />, label: 'Likes' },
              { icon: <FiSettings size={20} />, label: 'Settings' },
            ]}
            magnification={56}
            distance={150}
            panelHeight={52}
            baseItemSize={36}
          />
        </div>
      </Suspense>
    ),
  },
];

const LiveDemo = () => {
  const colors = useDemoColors();

  return (
    <section className="ln-demo-section">
      <div className="ln-demo-inner">
        <h2 className="ln-demo-title">See them in action</h2>

        <div className="ln-demo-grid">
          {CARDS.map((card, i) => (
            <motion.div
              key={card.component}
              className={`ln-demo-card ln-demo-card--span-${card.span}${card.tall ? ' ln-demo-card--tall' : ''}`}
              initial={{ opacity: 0, y: 24 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true, margin: '-60px' }}
              transition={{ duration: 0.5, delay: i * 0.07, ease: [0.21, 0.47, 0.32, 0.98] }}
            >
              <Link to={card.href} className="ln-demo-card-link">
                <div className="ln-demo-card-visual">{card.render(colors)}</div>
                <div className="ln-demo-card-overlay">
                  <span className="ln-demo-card-category">{card.category}</span>
                  <span className="ln-demo-card-name">{card.component}</span>
                </div>
              </Link>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default LiveDemo;
