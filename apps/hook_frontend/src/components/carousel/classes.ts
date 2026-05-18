import { createClasses } from 'src/theme/create-classes';

// ----------------------------------------------------------------------

export const carouselClasses = {
  root: createClasses('carousel__root'),
  container: createClasses('carousel__container'),
  dots: {
    root: createClasses('carousel__dots__root'),
    item: createClasses('carousel__dot__item'),
    itemSelected: createClasses('carousel__dot__selected'),
  },
  arrows: {
    root: createClasses('carousel__arrows__root'),
    label: createClasses('carousel__arrows__label'),
    prev: createClasses('carousel__arrow__prev'),
    next: createClasses('carousel__arrow__next'),
    svg: createClasses('carousel__arrows__svg'),
  },
  slide: {
    root: createClasses('carousel__slide__root'),
  },
};
