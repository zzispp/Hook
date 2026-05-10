'use client';

import type { ApiPermission, MenuItem as RbacMenuItem } from 'src/types/rbac';
import type { UseTableReturn, TableHeadCellProps } from 'src/components/table';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/components/table';

import { MethodLabel, EnabledLabel, TableLoadingRows, ManagementTableHead } from './shared';

type ApiMenuGroup = {
  menuId: string;
  menuCode: string;
  menuPath?: string;
  menuTitle: string;
  apis: ApiPermission[];
};

type Props = {
  apis: ApiPermission[];
  loading: boolean;
  menus: RbacMenuItem[];
  table: UseTableReturn;
  tableHead: TableHeadCellProps[];
  total: number;
  onDelete: (api: ApiPermission) => void;
  onEdit: (api: ApiPermission) => void;
};

const UNBOUND_MENU_ID = '__unbound__';
const UNBOUND_MENU_CODE = 'unbound_menu';

export function ApiManagementTable({
  apis,
  loading,
  menus,
  table,
  tableHead,
  total,
  onDelete,
  onEdit,
}: Props) {
  const { t } = useTranslate('admin');
  const groupedApis = useMemo(() => apiRowsByMenu(apis, menus), [apis, menus]);

  return (
    <>
      <Scrollbar>
        <Table sx={{ minWidth: 980 }}>
          <ManagementTableHead head={tableHead} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
            ) : (
              groupedApis.map((group) => [
                <ApiMenuHeaderRow key={`menu-${group.menuId}`} group={group} />,
                ...group.apis.map((row) => (
                  <ApiTableRow
                    key={`${group.menuId}-${row.id}`}
                    row={row}
                    onDelete={onDelete}
                    onEdit={onEdit}
                  />
                )),
              ])
            )}

            <TableNoData title={t('common.noData')} notFound={!loading && apis.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>

      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </>
  );
}

function ApiMenuHeaderRow({ group }: { group: ApiMenuGroup }) {
  const { t } = useTranslate('admin');
  const title = group.menuId === UNBOUND_MENU_ID ? t(`api.${group.menuCode}`) : group.menuTitle;

  return (
    <TableRow>
      <TableCell colSpan={6} sx={{ bgcolor: 'background.neutral', py: 1.25 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Iconify width={18} icon="solar:file-bold-duotone" />
          <Typography variant="subtitle2">{title}</Typography>
          {group.menuPath ? (
            <Typography variant="caption" sx={{ color: 'text.secondary', fontFamily: 'monospace' }}>
              {group.menuPath}
            </Typography>
          ) : null}
          <Typography variant="caption" sx={{ color: 'text.secondary' }}>
            {t('api.itemsCount', { count: group.apis.length })}
          </Typography>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function ApiTableRow({
  row,
  onDelete,
  onEdit,
}: {
  row: ApiPermission;
  onDelete: (api: ApiPermission) => void;
  onEdit: (api: ApiPermission) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell>
        <Box sx={{ display: 'flex', alignItems: 'center', pl: 3 }}>
          <Box component="span" sx={{ mr: 1, width: 10, height: 1, bgcolor: 'divider' }} />
          <MethodLabel method={row.method} />
        </Box>
      </TableCell>
      <TableCell>{row.name}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.path_pattern}</TableCell>
      <TableCell>
        <EnabledLabel enabled={row.enabled} />
      </TableCell>
      <TableCell align="right">
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('common.edit')}>
            <IconButton onClick={() => onEdit(row)}>
              <Iconify icon="solar:pen-bold" />
            </IconButton>
          </Tooltip>
          <Tooltip title={t('common.delete')}>
            <IconButton color="error" onClick={() => onDelete(row)}>
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}

function apiRowsByMenu(apis: ApiPermission[], menus: RbacMenuItem[]) {
  const menuById = new Map(menus.map((menu) => [menu.id, menu]));
  const menuOrder = new Map(menus.map((menu, index) => [menu.id, index]));
  const groupMap = new Map<string, ApiPermission[]>();

  for (const api of apis) {
    const menuIds = api.menu_item_ids.length ? api.menu_item_ids : [UNBOUND_MENU_ID];
    for (const menuId of menuIds) {
      groupMap.set(menuId, [...(groupMap.get(menuId) ?? []), api]);
    }
  }

  return Array.from(groupMap.entries())
    .sort(([left], [right]) => groupSortValue(left, menuOrder) - groupSortValue(right, menuOrder))
    .map(([menuId, groupApis]) => apiMenuGroup(menuId, groupApis, menuById));
}

function apiMenuGroup(
  menuId: string,
  apis: ApiPermission[],
  menuById: Map<string, RbacMenuItem>
): ApiMenuGroup {
  const menu = menuById.get(menuId);

  return {
    menuId,
    menuCode: menuId === UNBOUND_MENU_ID ? UNBOUND_MENU_CODE : (menu?.code ?? menuId),
    menuPath: menu?.path,
    menuTitle: menu?.title ?? menuId,
    apis: [...apis].sort(
      (left, right) => left.name.localeCompare(right.name) || left.code.localeCompare(right.code)
    ),
  };
}

function groupSortValue(menuId: string, menuOrder: Map<string, number>) {
  return menuId === UNBOUND_MENU_ID
    ? Number.MAX_SAFE_INTEGER
    : menuOrder.get(menuId) ?? Number.MAX_SAFE_INTEGER - 1;
}
