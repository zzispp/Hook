'use client';

import type { ApiPermission, MenuItem as RbacMenuItem } from 'src/types/rbac';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
import Checkbox from '@mui/material/Checkbox';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { MethodLabel } from './shared';

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

type ApiTransferListProps = {
  apis: ApiPermission[];
  selectedApiIds: string[];
  onSelectedApiIdsChange: (value: string[]) => void;
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

  return (
    <Dialog fullWidth maxWidth="lg" open={!!menu} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.menuApiPermissions', {
          name: menu?.title ?? '',
        })}
      </DialogTitle>
      <DialogContent>
        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
        ) : (
          <ApiTransferList
            apis={apis}
            selectedApiIds={selectedApiIds}
            onSelectedApiIdsChange={onSelectedApiIdsChange}
          />
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

function ApiTransferList({ apis, selectedApiIds, onSelectedApiIdsChange }: ApiTransferListProps) {
  const { t } = useTranslate('admin');
  const [checkedIds, setCheckedIds] = useState<string[]>([]);
  const selectedIdSet = useMemo(() => new Set(selectedApiIds), [selectedApiIds]);
  const availableApis = useMemo(() => apis.filter((api) => !selectedIdSet.has(api.id)), [apis, selectedIdSet]);
  const selectedApis = useMemo(() => apis.filter((api) => selectedIdSet.has(api.id)), [apis, selectedIdSet]);
  const availableIds = useMemo(() => availableApis.map((api) => api.id), [availableApis]);
  const selectedIds = useMemo(() => selectedApis.map((api) => api.id), [selectedApis]);
  const checkedAvailableIds = intersection(checkedIds, availableIds);
  const checkedSelectedIds = intersection(checkedIds, selectedIds);

  const moveToSelected = useCallback(() => {
    onSelectedApiIdsChange(uniqueByApiOrder(apis, [...selectedApiIds, ...checkedAvailableIds]));
    setCheckedIds(not(checkedIds, checkedAvailableIds));
  }, [apis, checkedAvailableIds, checkedIds, onSelectedApiIdsChange, selectedApiIds]);

  const moveToAvailable = useCallback(() => {
    onSelectedApiIdsChange(selectedApiIds.filter((id) => !checkedSelectedIds.includes(id)));
    setCheckedIds(not(checkedIds, checkedSelectedIds));
  }, [checkedIds, checkedSelectedIds, onSelectedApiIdsChange, selectedApiIds]);

  return (
    <Box
      sx={{
        pt: 1,
        gap: 2,
        display: 'grid',
        alignItems: 'center',
        gridTemplateColumns: { xs: '1fr', md: 'minmax(0, 1fr) auto minmax(0, 1fr)' },
      }}
    >
      <ApiTransferColumn
        apis={availableApis}
        checkedIds={checkedIds}
        title={t('apiTransfer.available')}
        onCheckedIdsChange={setCheckedIds}
      />
      <ApiTransferControls
        moveToSelected={moveToSelected}
        moveToAvailable={moveToAvailable}
        disableMoveToSelected={checkedAvailableIds.length === 0}
        disableMoveToAvailable={checkedSelectedIds.length === 0}
      />
      <ApiTransferColumn
        apis={selectedApis}
        checkedIds={checkedIds}
        title={t('apiTransfer.bound')}
        onCheckedIdsChange={setCheckedIds}
      />
    </Box>
  );
}

function ApiTransferColumn({
  apis,
  checkedIds,
  title,
  onCheckedIdsChange,
}: {
  apis: ApiPermission[];
  checkedIds: string[];
  title: string;
  onCheckedIdsChange: (value: string[]) => void;
}) {
  const { t } = useTranslate('admin');
  const ids = apis.map((api) => api.id);
  const checkedCount = intersection(checkedIds, ids).length;
  const toggleAll = () => {
    onCheckedIdsChange(checkedCount === ids.length ? not(checkedIds, ids) : union(checkedIds, ids));
  };

  return (
    <Card variant="outlined" sx={{ minWidth: 0 }}>
      <Box sx={{ px: 1, py: 1.5, gap: 0.5, display: 'flex', alignItems: 'center' }}>
        <Checkbox
          disabled={ids.length === 0}
          checked={ids.length > 0 && checkedCount === ids.length}
          indeterminate={checkedCount > 0 && checkedCount < ids.length}
          onChange={toggleAll}
        />
        <Box sx={{ minWidth: 0 }}>
          <Typography variant="subtitle2">{title}</Typography>
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {t('apiTransfer.selectedCount', { selected: checkedCount, total: ids.length })}
          </Typography>
        </Box>
      </Box>
      <Divider />
      <List dense component="div" role="list" sx={{ height: 420, overflow: 'auto', py: 0.5 }}>
        {apis.length === 0 ? (
          <Box sx={{ height: 1, display: 'flex', alignItems: 'center', justifyContent: 'center', color: 'text.secondary' }}>
            {t('common.noData')}
          </Box>
        ) : (
          apis.map((api) => (
            <ApiTransferItem
              key={api.id}
              api={api}
              checked={checkedIds.includes(api.id)}
              onToggle={() => onCheckedIdsChange(toggleValue(checkedIds, api.id))}
            />
          ))
        )}
      </List>
    </Card>
  );
}

function ApiTransferItem({
  api,
  checked,
  onToggle,
}: {
  api: ApiPermission;
  checked: boolean;
  onToggle: () => void;
}) {
  return (
    <ListItemButton role="listitem" onClick={onToggle} sx={{ px: 1, gap: 0.5 }}>
      <Checkbox disableRipple checked={checked} tabIndex={-1} />
      <ListItemText
        primary={
          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', minWidth: 0 }}>
            <MethodLabel method={api.method} />
            <Typography variant="body2" noWrap>
              {api.name}
            </Typography>
          </Box>
        }
        secondary={
          <Typography variant="caption" sx={{ color: 'text.secondary', fontFamily: 'monospace' }} noWrap>
            {api.path_pattern}
          </Typography>
        }
      />
    </ListItemButton>
  );
}

function ApiTransferControls({
  disableMoveToAvailable,
  disableMoveToSelected,
  moveToAvailable,
  moveToSelected,
}: {
  disableMoveToAvailable: boolean;
  disableMoveToSelected: boolean;
  moveToAvailable: () => void;
  moveToSelected: () => void;
}) {
  return (
    <Box sx={{ gap: 1, display: 'flex', flexDirection: { xs: 'row', md: 'column' }, justifyContent: 'center' }}>
      <Button color="inherit" variant="outlined" size="small" disabled={disableMoveToSelected} onClick={moveToSelected}>
        <Iconify width={18} icon="eva:arrow-ios-forward-fill" />
      </Button>
      <Button color="inherit" variant="outlined" size="small" disabled={disableMoveToAvailable} onClick={moveToAvailable}>
        <Iconify width={18} icon="eva:arrow-ios-back-fill" />
      </Button>
    </Box>
  );
}

function not(left: string[], right: string[]) {
  return left.filter((value) => !right.includes(value));
}

function intersection(left: string[], right: string[]) {
  return left.filter((value) => right.includes(value));
}

function union(left: string[], right: string[]) {
  return [...left, ...not(right, left)];
}

function toggleValue(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

function uniqueByApiOrder(apis: ApiPermission[], ids: string[]) {
  const idSet = new Set(ids);
  return apis.filter((api) => idSet.has(api.id)).map((api) => api.id);
}
