import { createClasses } from 'src/theme/create-classes';

// ----------------------------------------------------------------------

export const dndClasses = {
  item: createClasses('dnd__item'),
  removeBtn: createClasses('dnd__remove__btn'),
  state: {
    idle: '--idle',
    over: '--over',
    flash: '--flash',
    preview: '--preview',
    dragging: '--dragging',
    draggingAndLeftSelf: '--dragging-and-left-self',
  },
} as const;
