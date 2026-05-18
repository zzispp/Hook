import type { NavListProps, NavSubListProps } from '../types';

import { useBoolean } from 'minimal-shared/hooks';
import { useRef, useEffect, useCallback } from 'react';
import { isEqualPath, isActiveLink, isExternalLink } from 'minimal-shared/utils';

import { usePathname } from 'src/routes/hooks';

import { useTranslate } from 'src/locales';

import { NavItem } from './nav-desktop-item';
import { Nav, NavLi, NavUl, NavDropdown } from '../components';
import { NavItemDashboard } from './nav-desktop-item-dashboard';

// ----------------------------------------------------------------------

export function NavList({ data, sx, ...other }: NavListProps) {
  const pathname = usePathname();
  const { t } = useTranslate('common');
  const navItemRef = useRef<HTMLButtonElement>(null);

  const isActive = isActiveLink(pathname, data.path, data.deepMatch ?? !!data.children);
  const title = data.titleKey ? t(data.titleKey) : data.title;

  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();

  useEffect(() => {
    if (open) {
      onClose();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  const handleOpenMenu = useCallback(() => {
    if (data.children) {
      onOpen();
    }
  }, [data.children, onOpen]);

  const renderNavItem = () => (
    <NavItem
      ref={navItemRef}
      // slots
      path={data.path}
      title={title}
      // state
      open={open}
      active={isActive}
      // options
      hasChild={!!data.children}
      externalLink={isExternalLink(data.path)}
      // action
      onMouseEnter={handleOpenMenu}
      onMouseLeave={onClose}
    />
  );

  const renderDropdown = () =>
    !!data.children && (
      <NavDropdown open={open} onMouseEnter={handleOpenMenu} onMouseLeave={onClose}>
        <Nav>
          <NavUl sx={{ gap: 3, flexDirection: 'row' }}>
            {data.children.map((list) => (
              <NavSubList key={list.subheader} subheader={list.subheader} data={list.items} />
            ))}
          </NavUl>
        </Nav>
      </NavDropdown>
    );

  return (
    <NavLi sx={sx} {...other}>
      {renderNavItem()}
      {renderDropdown()}
    </NavLi>
  );
}

// ----------------------------------------------------------------------

function NavSubList({ data, subheader, sx, ...other }: NavSubListProps) {
  const pathname = usePathname();
  const { t } = useTranslate('common');

  const isDashboard = subheader === 'Dashboard';
  const label = subheader === 'Dashboard' ? t('nav.dashboard') : subheader;

  return (
    <NavLi
      sx={[
        () => ({
          flexGrow: 1,
          flexBasis: 'auto',
          flexShrink: isDashboard ? 1 : 0,
          ...(isDashboard && { maxWidth: 560 }),
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <NavUl>
        <NavLi
          sx={(theme) => ({
            mb: 0.75,
            typography: 'overline',
            fontSize: theme.typography.pxToRem(11),
          })}
        >
          {label}
        </NavLi>

        {data.map((item) =>
          isDashboard ? (
            <NavLi key={item.title} sx={{ mt: 0.75 }}>
              <NavItemDashboard path={item.path} />
            </NavLi>
          ) : (
            <NavLi key={item.title} sx={{ mt: 0.75 }}>
              <NavItem
                subItem
                title={item.titleKey ? t(item.titleKey) : item.title}
                path={item.path}
                active={isEqualPath(item.path, pathname)}
              />
            </NavLi>
          )
        )}
      </NavUl>
    </NavLi>
  );
}
