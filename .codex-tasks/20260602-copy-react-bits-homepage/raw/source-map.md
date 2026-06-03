# Source Map

## Hook Entry

- Current route: `apps/hook_frontend/src/app/(home)/page.tsx`
- Current homepage imports: `src/components/hook-rs-landing/*`
- Baseline lint before this migration fails because `hook-rs-landing` references missing packages and has stale animation hook issues.

## React Bits Entry

- Source route: `/Users/bubu/ZwjProjects/react-bits/src/pages/LandingPage.jsx`
- Source sections:
  - `components/landingnew/Navbar/Navbar.jsx`
  - `components/landingnew/Hero/Hero.jsx`
  - `components/landingnew/Features/Features.jsx`
  - `components/landingnew/Testimonials/Testimonials.jsx`
  - `components/landingnew/LiveDemo/LiveDemo.jsx`
  - `components/landingnew/QuickStart/QuickStart.jsx`
  - `components/landingnew/Sponsors/Sponsors.jsx`
  - `components/landingnew/CTA/CTA.jsx`
  - `components/landingnew/Footer/Footer.jsx`
  - `components/landingnew/LandingLoader/LandingLoader.jsx`

## Required Support

- Content demos for homepage cards:
  - `content/Backgrounds/ShapeGrid`
  - `content/TextAnimations/ShinyText`
  - `content/Animations/MagicRings`
  - `content/Components/Dock`
- Shared support:
  - `components/common/SVGComponents.jsx`
  - `hooks/useScrollToTop.js`
  - `hooks/useStars.js`
  - `hooks/useInstallation.js`
  - `constants/Site.js`
  - `constants/Sponsors.js`
  - minimal local context providers for installation, search, and options.
- Static assets:
  - `src/assets/logos/*`
  - `src/assets/icons/{js,ts,css,tw}.svg`
  - `public/assets/sponsors/*`
  - `public/assets/sounds/{click-004,switch-007}.mp3`

## Next.js Adaptations

- Replace React Router `Link`/`useLocation` with a local Next.js-compatible adapter.
- Import the landing CSS from the root app layout through one global stylesheet, not from individual components.
- Replace the old `hook-rs-landing` homepage entry.
