import type { AdminT } from './shared';
import type { TableHeadCellProps } from 'src/components/table';

import { withStickyActionHeadCell } from 'src/components/table';

export function apiTableHead(t: AdminT): TableHeadCellProps[] {
  return [
    { id: 'method', label: t('common.method'), width: 110 },
    { id: 'name', label: t('common.name'), width: 220 },
    { id: 'code', label: t('common.code'), width: 220 },
    { id: 'path_pattern', label: t('fields.pathPattern') },
    { id: 'enabled', label: t('common.status'), width: 120 },
    withStickyActionHeadCell({ id: 'actions', label: t('common.actions'), width: 96, align: 'left' }),
  ];
}
