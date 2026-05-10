'use client';

import type { Role, ApiPermission, MenuItem as RbacMenuItem } from 'src/types/rbac';

import { useState } from 'react';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import List from '@mui/material/List';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Checkbox from '@mui/material/Checkbox';
import ListItem from '@mui/material/ListItem';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';

import { useTranslate } from 'src/locales/use-locales';

import { Scrollbar } from 'src/components/scrollbar';

import { MethodLabel } from './shared';

type PermissionTab = 'menus' | 'apis';

type Props = {
  apis: ApiPermission[];
  loading: boolean;
  menus: RbacMenuItem[];
  role: Role | null;
  selectedApis: string[];
  selectedMenus: string[];
  submitting: boolean;
  onClose: () => void;
  onSelectedApisChange: (value: string[]) => void;
  onSelectedMenusChange: (value: string[]) => void;
  onSubmit: () => void;
};

export function RolePermissionDialog({
  apis,
  loading,
  menus,
  role,
  selectedApis,
  selectedMenus,
  submitting,
  onClose,
  onSelectedApisChange,
  onSelectedMenusChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState<PermissionTab>('menus');

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.rolePermissions', {
          name: role?.name ?? '',
        })}
      </DialogTitle>
      <DialogContent>
        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
        ) : (
          <>
            <Tabs value={tab} onChange={(_event, value: PermissionTab) => setTab(value)} sx={{ mb: 2 }}>
              <Tab value="menus" label={t('common.menus')} />
              <Tab value="apis" label={t('common.apis')} />
            </Tabs>
            {tab === 'menus' ? (
              <MenuPermissionList
                menus={menus}
                selectedMenus={selectedMenus}
                onSelectedMenusChange={onSelectedMenusChange}
              />
            ) : (
              <ApiPermissionList
                apis={apis}
                selectedApis={selectedApis}
                onSelectedApisChange={onSelectedApisChange}
              />
            )}
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('actions.savePermissions')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function MenuPermissionList({
  menus,
  selectedMenus,
  onSelectedMenusChange,
}: {
  menus: RbacMenuItem[];
  selectedMenus: string[];
  onSelectedMenusChange: (value: string[]) => void;
}) {
  return (
    <Scrollbar sx={{ maxHeight: 520 }}>
      <List disablePadding>
        {menus.map((menu) => (
          <ListItem key={menu.id} disablePadding>
            <ListItemButton onClick={() => onSelectedMenusChange(toggleValue(selectedMenus, menu.id))}>
              <Checkbox edge="start" checked={selectedMenus.includes(menu.id)} tabIndex={-1} />
              <ListItemText primary={menu.title} secondary={`${menu.code} · ${menu.path}`} />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
    </Scrollbar>
  );
}

function ApiPermissionList({
  apis,
  selectedApis,
  onSelectedApisChange,
}: {
  apis: ApiPermission[];
  selectedApis: string[];
  onSelectedApisChange: (value: string[]) => void;
}) {
  return (
    <Scrollbar sx={{ maxHeight: 520 }}>
      <List disablePadding>
        {apis.map((api) => (
          <ListItem key={api.id} disablePadding>
            <ListItemButton onClick={() => onSelectedApisChange(toggleValue(selectedApis, api.id))}>
              <Checkbox edge="start" checked={selectedApis.includes(api.id)} tabIndex={-1} />
              <ListItemText
                primary={
                  <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                    <MethodLabel method={api.method} />
                    <span>{api.name}</span>
                  </Box>
                }
                secondary={api.path_pattern}
              />
            </ListItemButton>
          </ListItem>
        ))}
      </List>
    </Scrollbar>
  );
}

function toggleValue(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
