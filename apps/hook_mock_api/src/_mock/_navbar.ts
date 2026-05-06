// ----------------------------------------------------------------------

export const _navItems = [
  {
    subheader: 'Marketing',
    items: [
      {
        title: 'Landing',
        path: '#landing',
        icon: 'icon.landing',
        info: ['info.landing', '+1'],
        roles: ['admin'],
        caption: 'Display only admin role',
      },
      {
        title: 'Services',
        path: '#services',
        icon: 'icon.services',
        roles: ['admin', 'user'],
      },
      {
        title: 'Blog',
        path: '#blog',
        icon: 'icon.blog',
        info: ['info.blog', '+2'],
        children: [
          {
            title: 'Item 1',
            path: '#blog/item-1',
            caption: 'Display caption',
            info: ['info.blog.item1', '+3'],
          },
          { title: 'Item 2', path: '#blog/item-2' },
        ],
      },
    ],
  },
  {
    subheader: 'Travel',
    items: [
      { title: 'About', path: '#about', icon: 'icon.about' },
      { title: 'Contact', path: '#contact', icon: 'icon.tour' },
      {
        title: 'Level',
        path: '#level',
        icon: 'icon.menu',
        children: [
          {
            title: 'Level 2a',
            path: '#level/2a',
            icon: 'icon.level2a',
            caption: 'This is the caption',
            children: [
              { title: 'Level 3a', path: '#level/2a/3a' },
              {
                title: 'Level 3b',
                path: '#level/2a/3b',
                children: [
                  { title: 'Level 4a', path: '#level/2a/3b/4a' },
                  { title: 'Level 4b', path: '#level/2a/3b/4b' },
                ],
              },
              { title: 'Level 3c', path: '#level/2a/3c' },
            ],
          },
          { title: 'Level 2b', path: '#level/2b', icon: 'icon.level2b' },
          { title: 'Level 2c', path: '#level/2c', icon: 'icon.level2c' },
        ],
      },
    ],
  },
];
