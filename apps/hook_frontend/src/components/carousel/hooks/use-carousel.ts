'use client';

import type { EmblaPluginType } from 'embla-carousel';
import type { CarouselOptions, UseCarouselReturn } from '../types';

import useEmblaCarousel from 'embla-carousel-react';

import { useTheme } from '@mui/material/styles';

import { useCarouselDots } from './use-carousel-dots';
import { useCarouselArrows } from './use-carousel-arrows';

// ----------------------------------------------------------------------

export function useCarousel(
  options?: CarouselOptions,
  plugins?: EmblaPluginType[]
): UseCarouselReturn {
  const theme = useTheme();

  const [mainRef, mainApi] = useEmblaCarousel({ ...options, direction: theme.direction }, plugins);

  const pluginNames = plugins?.map((plugin) => plugin.name);

  const dots = useCarouselDots(mainApi);
  const arrows = useCarouselArrows(mainApi);

  const mergedOptions = { ...options, ...mainApi?.internalEngine().options };

  return {
    options: mergedOptions,
    pluginNames,
    mainRef,
    mainApi,
    arrows,
    dots,
  };
}
