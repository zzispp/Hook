'use client';

import type { Theme, SxProps } from '@mui/material/styles';
import type { IconButtonProps } from '@mui/material/IconButton';
import type { NavSectionProps, NavItemDataProps } from 'src/components/nav-section';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Avatar from '@mui/material/Avatar';
import Drawer from '@mui/material/Drawer';
import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import ListSubheader from '@mui/material/ListSubheader';

import { paths } from 'src/routes/paths';

import { useTranslate } from 'src/locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { AnimateBorder } from 'src/components/animate';
import { navItemKey, navGroupKey, createNavItem } from 'src/components/nav-section';

import { useMockedUser } from 'src/auth/hooks';

import { AccountButton } from './account-button';
import { SignOutButton } from './sign-out-button';

// ----------------------------------------------------------------------

export type AccountDrawerProps = IconButtonProps & {
  data?: NavSectionProps['data'];
};

export function AccountDrawer({ data = [], sx, ...other }: AccountDrawerProps) {
  const { user } = useMockedUser();
  const { t } = useTranslate('common');

  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();

  const renderAvatar = () => (
    <AnimateBorder
      sx={{ mb: 2, p: '6px', width: 96, height: 96, borderRadius: '50%' }}
      slotProps={{
        primaryBorder: { size: 120, sx: { color: 'primary.main' } },
      }}
    >
      <Avatar src={user?.photoURL} alt={user?.displayName} sx={{ width: 1, height: 1 }}>
        {user?.displayName?.charAt(0).toUpperCase()}
      </Avatar>
    </AnimateBorder>
  );

  const renderList = () => (
    <MenuList disablePadding sx={menuListStyles}>
      <AccountDrawerNavItem
        item={{
          title: t('profile.menu'),
          path: paths.dashboard.profile,
          icon: 'solar:user-id-bold',
        }}
        onClose={onClose}
      />
      {data.flatMap((group) => renderNavGroup(group, onClose))}
    </MenuList>
  );

  return (
    <>
      <AccountButton
        onClick={onOpen}
        photoURL={user?.photoURL}
        displayName={user?.displayName}
        sx={sx}
        {...other}
      />

      <Drawer
        open={open}
        onClose={onClose}
        anchor="right"
        slotProps={{
          backdrop: { invisible: true },
          paper: { sx: { width: 320 } },
        }}
      >
        <IconButton
          onClick={onClose}
          sx={{
            top: 12,
            left: 12,
            zIndex: 9,
            position: 'absolute',
          }}
        >
          <Iconify icon="mingcute:close-line" />
        </IconButton>

        <Scrollbar>
          <Box
            sx={{
              pt: 8,
              display: 'flex',
              alignItems: 'center',
              flexDirection: 'column',
            }}
          >
            {renderAvatar()}

            <Typography variant="subtitle1" noWrap sx={{ mt: 2 }}>
              {user?.displayName}
            </Typography>

            <Typography variant="body2" sx={{ color: 'text.secondary', mt: 0.5 }} noWrap>
              {user?.email}
            </Typography>
          </Box>

          {renderList()}
        </Scrollbar>

        <Box sx={{ p: 2.5 }}>
          <SignOutButton onClose={onClose} />
        </Box>
      </Drawer>
    </>
  );
}

function renderNavGroup(group: NavSectionProps['data'][number], onClose: () => void) {
  const groupKey = navGroupKey(group);
  const header = group.subheader ? (
    <ListSubheader key={`${groupKey}-subheader`} disableSticky>
      {group.subheader}
    </ListSubheader>
  ) : null;
  const items = flattenNavItems(group.items).map((item) => (
    <AccountDrawerNavItem key={navItemKey(item)} item={item} onClose={onClose} />
  ));

  return header ? [header, ...items] : items;
}

function AccountDrawerNavItem({
  item,
  onClose,
}: {
  item: NavItemDataProps;
  onClose: () => void;
}) {
  const navItem = createNavItem({ path: item.path, icon: item.icon, info: item.info });

  return (
    <MenuItem>
      <Link
        color="inherit"
        underline="none"
        onClick={onClose}
        sx={{
          p: 1,
          width: 1,
          display: 'flex',
          typography: 'body2',
          alignItems: 'center',
          color: 'text.secondary',
          '& svg': { width: 24, height: 24 },
          '&:hover': { color: 'text.primary' },
        }}
        {...navItem.baseProps}
      >
        {navItem.renderIcon}

        <Box component="span" sx={{ ml: 2, minWidth: 0, flex: '1 1 auto' }}>
          {item.title}
        </Box>

        {navItem.renderInfo && <Box sx={{ ml: 1 }}>{navItem.renderInfo}</Box>}
      </Link>
    </MenuItem>
  );
}

function flattenNavItems(items: readonly NavItemDataProps[]): NavItemDataProps[] {
  return items.flatMap((item) =>
    item.children?.length ? [item, ...flattenNavItems(item.children)] : [item]
  );
}

const menuListStyles: SxProps<Theme> = [
  (theme) => ({
    mt: 3,
    py: 3,
    px: 2.5,
    borderTop: `dashed 1px ${theme.vars.palette.divider}`,
    borderBottom: `dashed 1px ${theme.vars.palette.divider}`,
    '& li': { p: 0 },
    '& .MuiListSubheader-root': {
      px: 1,
      mb: 0.75,
      color: 'text.disabled',
      bgcolor: 'transparent',
      typography: 'overline',
      fontSize: theme.typography.pxToRem(11),
    },
  }),
];
