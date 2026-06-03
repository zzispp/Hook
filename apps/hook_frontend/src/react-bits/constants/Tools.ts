import type { IconifyName } from 'src/components/iconify';

type Tool = {
  readonly id: string;
  readonly label: string;
  readonly icon: IconifyName;
  readonly path: string;
  readonly description: string;
};

export const TOOLS: readonly Tool[] = [
  {
    id: 'background-studio',
    label: 'Background Studio',
    icon: 'solar:palette-bold',
    path: '/tools/background-studio',
    description: 'Explore animated backgrounds for your projects. Choose from various effects and customize as you like. Export as video, image, or code or share your creations as URLs.'
  },
  {
    id: 'shape-magic',
    label: 'Shape Magic',
    icon: 'solar:atom-bold-duotone',
    path: '/tools/shape-magic',
    description: 'Tool for automagically creating inner rounded corners between shapes of different sizes. Export as code or SVG.'
  },
  {
    id: 'texture-lab',
    label: 'Texture Lab',
    icon: 'solar:gallery-wide-bold',
    path: '/tools/texture-lab',
    description: 'Apply effects to your images and export the results. Add noise, dithering, halftone, ASCII art, and more. Save your presets for sharing or future use.'
  }
];
