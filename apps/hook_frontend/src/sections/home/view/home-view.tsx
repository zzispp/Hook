'use client';

import Stack from '@mui/material/Stack';

import { BackToTopButton } from 'src/components/animate/back-to-top-button';
import { ScrollProgress, useScrollProgress } from 'src/components/animate/scroll-progress';

import { HomeHero } from '../home-hero';
import { HomeUseCases } from '../home-use-cases';
import { HomeOperations } from '../home-operations';
import { HomeGatewayCTA } from '../home-gateway-cta';
import { HomeGatewayFAQs } from '../home-gateway-faqs';
import { HomeProviderMesh } from '../home-provider-mesh';
import { HomeGatewayOverview } from '../home-gateway-overview';

// ----------------------------------------------------------------------

export function HomeView() {
  const pageProgress = useScrollProgress();

  return (
    <>
      <ScrollProgress
        variant="linear"
        progress={pageProgress.scrollYProgress}
        sx={[(theme) => ({ position: 'fixed', zIndex: theme.zIndex.appBar + 1 })]}
      />

      <BackToTopButton />

      <HomeHero />

      <Stack sx={{ position: 'relative', bgcolor: 'background.default' }}>
        <HomeGatewayOverview />

        <HomeProviderMesh />

        <HomeOperations />

        <HomeUseCases />

        <HomeGatewayFAQs />

        <HomeGatewayCTA />
      </Stack>
    </>
  );
}
