import type { Theme, SxProps } from '@mui/material/styles';
import type { TableHeadCellProps } from './table-head-custom';

export const TABLE_STICKY_ACTION_CELL_WIDTH = 180;

const ACTION_CELL_Z_INDEX = 2;
const ACTION_HEAD_CELL_Z_INDEX = 3;

const stickyActionBorder = (theme: Theme) => `1px solid ${theme.vars.palette.divider}`;

const stickyActionDividerSx = {
  borderLeft: stickyActionBorder,
};

export const tableStickyActionCellSx: SxProps<Theme> = {
  position: 'sticky',
  right: 0,
  zIndex: ACTION_CELL_Z_INDEX,
  bgcolor: 'background.paper',
  whiteSpace: 'nowrap',
  ...stickyActionDividerSx,
};

export const tableStickyActionHeadCellSx: SxProps<Theme> = {
  position: 'sticky',
  right: 0,
  zIndex: ACTION_HEAD_CELL_Z_INDEX,
  bgcolor: 'background.neutral',
  backgroundImage: (theme: Theme) =>
    `linear-gradient(to bottom, ${theme.vars.palette.background.neutral}, ${theme.vars.palette.background.neutral})`,
  whiteSpace: 'nowrap',
  ...stickyActionDividerSx,
};

export function withStickyActionHeadCell(cell: TableHeadCellProps): TableHeadCellProps {
  const sx = Array.isArray(cell.sx)
    ? [...cell.sx, tableStickyActionHeadCellSx]
    : [cell.sx, tableStickyActionHeadCellSx];

  return {
    ...cell,
    align: 'left',
    sx: sx.filter(Boolean) as SxProps<Theme>,
  };
}
