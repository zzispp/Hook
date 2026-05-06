import type { NavBasicProps } from 'src/components/nav-basic';
import type { NavSectionProps } from 'src/components/nav-section';

import { CONFIG } from 'src/global-config';

import { Label } from 'src/components/label';
import { SvgColor } from 'src/components/svg-color';

// ----------------------------------------------------------------------

export const NAV_BASIC_ITEMS: NavBasicProps['data'] = [
  {
    title: 'Home',
    path: '#',
    icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-analytics.svg`} />,
  },
  {
    title: 'Page',
    path: '/components',
    caption: 'The standard Lorem Ipsum passage, used since the 1500s.',
    icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-banking.svg`} />,
    info: <Label color="info">+2</Label>,
    children: [
      {
        title: 'What is Lorem Ipsum?',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-booking.svg`} />,
        info: '+3',
        children: [
          { title: 'Page 1.1', path: '#' },
          { title: 'Page 1.2', path: '#', disabled: true },
        ],
      },
      {
        title: 'Page 2',
        path: '/components/extra',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-chat.svg`} />,
        caption: 'Lorem Ipsum is simply dummy text of the printing and typesetting industry.',
        children: [
          { title: 'Page 2.1', path: '#' },
          { title: 'Page 2.2', path: '#' },
          {
            title: 'Page 2.3',
            path: '/components/extra/navigation-bar',
            children: [
              { title: 'Page 2.3.1', path: '#' },
              { title: 'Page 2.3.2', path: '/components/extra/navigation-bar' },
              { title: 'Page 2.3.3', path: '#' },
            ],
          },
        ],
      },
      {
        title: 'Page 3',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-lock.svg`} />,
      },
    ],
  },
  {
    title: 'Blog',
    path: '#',
    icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-mail.svg`} />,
    children: [
      {
        title: 'Post 1',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-booking.svg`} />,
        caption: 'This is the caption',
        info: '+3',
      },
      {
        title: 'Post 2',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-chat.svg`} />,
      },
      {
        title: 'Post 3',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-lock.svg`} />,
      },
    ],
  },
  {
    title: 'Contact',
    path: '#',
    icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-user.svg`} />,
    disabled: true,
  },
  {
    title: 'External',
    path: 'https://www.google.com/',
    icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-tour.svg`} />,
  },
];

// ----------------------------------------------------------------------

/**
 * Input nav data is an array of navigation section items used to define the structure and content of a navigation bar.
 * Each section contains a subheader and an array of items, which can include nested children items.
 *
 * Each item can have the following properties:
 * - `title`: The title of the navigation item.
 * - `path`: The URL path the item links to.
 * - `icon`: An optional icon component to display alongside the title.
 * - `info`: Optional additional information to display, such as a label.
 * - `allowedRoles`: An optional array of roles that are allowed to see the item.
 * - `caption`: An optional caption to display below the title.
 * - `children`: An optional array of nested navigation items.
 * - `disabled`: An optional boolean to disable the item.
 */

/**
 * Permissions can be set for each item by using the `allowedRoles` property.
 * - If `allowedRoles` is not set (default), all roles can see the item.
 * - If `allowedRoles` is an empty array `[]`, no one can see the item.
 * - If `allowedRoles` contains specific roles, only those roles can see the item.
 *
 * Examples:
 * - `allowedRoles: ['user']` - only users with the 'user' role can see this item.
 * - `allowedRoles: ['admin']` - only users with the 'admin' role can see this item.
 * - `allowedRoles: ['admin', 'manager']` - only users with the 'admin' or 'manager' roles can see this item.
 *
 * Combine with the `checkPermissions` prop to build conditional expressions.
 * Example usage can be found in: src/sections/_examples/extra/navigation-bar-view/nav-vertical.{jsx | tsx}
 */

export const NAV_SECTION_ITEMS: NavSectionProps['data'] = [
  {
    subheader: 'Marketing',
    items: [
      {
        title: 'Landing',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-dashboard.svg`} />,
        info: <Label color="error">+2 </Label>,
      },
      {
        title: 'Services',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-analytics.svg`} />,
        allowedRoles: ['admin'],
        caption: 'Only admin can see this item.',
      },
      {
        title: 'Blog',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-blog.svg`} />,
        info: <Label color="info">+3 </Label>,
        allowedRoles: ['admin', 'manager'],
        caption: 'Only admin / manager can see this item.',
        children: [
          {
            title: 'Item 1',
            path: '#',
            caption: 'Display caption',
            info: '+2',
          },
          { title: 'Item 2', path: '#' },
        ],
      },
    ],
  },
  {
    subheader: 'Travel',
    items: [
      {
        title: 'About',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-user.svg`} />,
        info: '+4',
      },
      {
        title: 'Contact',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-tour.svg`} />,
        disabled: true,
      },
      {
        title: 'Level',
        path: '/components',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-menu-item.svg`} />,
        children: [
          {
            title: 'Level 2a',
            path: '/components/extra',
            icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-chat.svg`} />,
            caption: 'Lorem Ipsum is simply dummy text of the printing and typesetting industry.',
            children: [
              { title: 'Level 3a', path: '#' },
              {
                title: 'Level 3b',
                path: '/components/extra/navigation-bar',
                children: [
                  { title: 'Level 4a', path: '#', disabled: true },
                  { title: 'Level 4b', path: '/components/extra/navigation-bar' },
                ],
              },
              { title: 'Level 3c', path: '#' },
            ],
          },
          {
            title: 'Level 2b',
            path: '#',
            icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-mail.svg`} />,
          },
          {
            title: 'Level 2c',
            path: '#',
            icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-calendar.svg`} />,
          },
        ],
      },
      {
        title: 'More',
        path: '#',
        icon: <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/ic-blank.svg`} />,
      },
    ],
  },
];
