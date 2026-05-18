'use client';

import type { CarouselOptions, CarouselSlideProps } from '../types';

import { mergeClasses } from 'minimal-shared/utils';

import { styled } from '@mui/material/styles';

import { getSlideSize } from '../utils';
import { carouselClasses } from '../classes';

// ----------------------------------------------------------------------

export function CarouselSlide({ sx, options, children, className, ...other }: CarouselSlideProps) {
  const slideSize = getSlideSize(options?.slidesToShow);

  return (
    <CarouselSlideRoot
      axis={options?.axis ?? 'x'}
      slideSpacing={options?.slideSpacing}
      className={mergeClasses([carouselClasses.slide.root, className])}
      sx={[{ flex: slideSize }, ...(Array.isArray(sx) ? sx : [sx])]}
      {...other}
    >
      {children}
    </CarouselSlideRoot>
  );
}

// ----------------------------------------------------------------------

const CarouselSlideRoot = styled('li', {
  shouldForwardProp: (prop: string) => !['axis', 'slideSpacing', 'sx'].includes(prop),
})<Pick<CarouselOptions, 'axis' | 'slideSpacing'>>(({ slideSpacing }) => ({
  display: 'block',
  position: 'relative',
  variants: [
    { props: { axis: 'x' }, style: { minWidth: 0, paddingLeft: slideSpacing } },
    { props: { axis: 'y' }, style: { minHeight: 0, paddingTop: slideSpacing } },
  ],
}));
