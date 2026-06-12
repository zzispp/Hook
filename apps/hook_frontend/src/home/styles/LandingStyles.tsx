'use client';

import GlobalStyles from '@mui/material/GlobalStyles';

import { LANDING_STYLE_CHUNKS } from './landing-style-chunks';

export function LandingStyles() {
  return <GlobalStyles styles={LANDING_STYLE_CHUNKS} />;
}
