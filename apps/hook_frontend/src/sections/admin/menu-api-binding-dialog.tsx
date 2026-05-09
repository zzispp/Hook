'use client';

import type { ApiPermission, MenuItem as RbacMenuItem } from 'src/types/rbac';

import { useCallback } from 'react';

import Box from '@mui/material/Box';
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

import {
  MethodLabel,
  translatedApiName,
  translatedApiGroup,
  translatedMenuItem,
} from './shared';

type Props = {
  apis: ApiPermission[];
  loading: boolean;
  menu: RbacMenuItem | null;
  selectedApiIds: string[];
  submitting: boolean;
  onClose: () => void;
  onSelectedApiIdsChange: (value: string[]) => void;
  onSubmit: () => void;
};

export function MenuApiBindingDialog({
  apis,
  loading,
  menu,
  selectedApiIds,
  submitting,
  onClose,
  onSelectedApiIdsChange,
  onSubmit,
}: Props) {
  const { t } = useTranslate('admin');
  const toggleApi = useCallback(
    (id: string) => {
      onSelectedApiIdsChange(toggleValue(selectedApiIds, id));
    },
    [onSelectedApiIdsChange, selectedApiIds]
  );

  return (
    <Dialog fullWidth maxWidth="md" open={!!menu} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.menuApiPermissions', {
          name: menu ? translatedMenuItem(menu, t) : '',
        })}
      </DialogTitle>
      <DialogContent>
        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
        ) : (
          <Scrollbar sx={{ maxHeight: 520 }}>
            <List disablePadding>
              {apis.map((api) => (
                <ListItem key={api.id} disablePadding>
                  <ListItemButton onClick={() => toggleApi(api.id)}>
                    <Checkbox edge="start" checked={selectedApiIds.includes(api.id)} tabIndex={-1} />
                    <ListItemText
                      primary={
                        <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                          <MethodLabel method={api.method} />
                          <span>{translatedApiName(api, t)}</span>
                        </Box>
                      }
                      secondary={`${translatedApiGroup(api.group, t)} · ${api.path_pattern}`}
                    />
                  </ListItemButton>
                </ListItem>
              ))}
            </List>
          </Scrollbar>
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

function toggleValue(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
