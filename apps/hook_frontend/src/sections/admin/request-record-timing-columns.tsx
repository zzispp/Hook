import type { Theme, SxProps } from '@mui/material/styles';
import type { RequestRecord } from 'src/types/provider';
import type { TableHeadCellProps } from 'src/components/table';
import type { RequestTimingMetric } from './request-record-timing';

import TableCell from '@mui/material/TableCell';

import { RequestRecordDurationText } from './request-record-duration-text';

type TimingColumn = Readonly<{
  metric: RequestTimingMetric;
  labelKey: string;
  width: number;
}>;

const TIMING_CELL_Z_INDEX = 2;
const TIMING_HEAD_CELL_Z_INDEX = 3;

const COMPACT_TIMING_COLUMNS: readonly TimingColumn[] = [
  { metric: 'first_output', labelKey: 'requestRecords.firstTokenShort', width: 112 },
  { metric: 'total_latency', labelKey: 'requestRecords.latencyShort', width: 120 },
];

const EXPANDED_TIMING_COLUMNS: readonly TimingColumn[] = [
  { metric: 'response_headers', labelKey: 'requestRecords.responseHeadersShort', width: 112 },
  { metric: 'first_sse_event', labelKey: 'requestRecords.firstSseEventShort', width: 112 },
  { metric: 'first_output', labelKey: 'requestRecords.firstOutputShort', width: 112 },
  { metric: 'total_latency', labelKey: 'requestRecords.totalLatencyShort', width: 120 },
];

export function requestRecordTimingHeadCells(
  t: (key: string) => string,
  expanded: boolean
): TableHeadCellProps[] {
  const columns = timingColumns(expanded);
  return columns.map((column) => ({
    id: column.metric,
    label: t(column.labelKey),
    width: column.width,
    sx: timingCellSx(column.metric, true, columns),
  }));
}

export function RequestRecordTimingCells({
  record,
  now,
  expanded,
}: {
  record: RequestRecord;
  now: number;
  expanded: boolean;
}) {
  const columns = timingColumns(expanded);
  return columns.map((column) => (
    <TableCell key={column.metric} sx={timingCellSx(column.metric, false, columns)}>
      <RequestRecordDurationText record={record} metric={column.metric} now={now} />
    </TableCell>
  ));
}

function timingColumns(expanded: boolean) {
  return expanded ? EXPANDED_TIMING_COLUMNS : COMPACT_TIMING_COLUMNS;
}

function timingCellSx(
  metric: RequestTimingMetric,
  head: boolean,
  columns: readonly TimingColumn[]
): SxProps<Theme> {
  const column = timingColumn(metric, columns);
  return {
    position: { xs: 'static', lg: 'sticky' },
    right: { lg: timingRightOffset(metric, columns) },
    zIndex: { lg: head ? TIMING_HEAD_CELL_Z_INDEX : TIMING_CELL_Z_INDEX },
    width: column.width,
    minWidth: column.width,
    maxWidth: column.width,
    whiteSpace: 'nowrap',
    bgcolor: head ? 'background.neutral' : 'background.paper',
    ...(head ? timingHeadBackgroundSx : {}),
    ...(metric === columns[0].metric ? timingDividerSx : {}),
  };
}

function timingRightOffset(metric: RequestTimingMetric, columns: readonly TimingColumn[]) {
  const index = columns.findIndex((column) => column.metric === metric);
  return columns.slice(index + 1).reduce((sum, column) => sum + column.width, 0);
}

function timingColumn(metric: RequestTimingMetric, columns: readonly TimingColumn[]) {
  return columns.find((column) => column.metric === metric) ?? columns[0];
}

const timingBorder = (theme: Theme) => `1px solid ${theme.vars.palette.divider}`;

const timingDividerSx = {
  borderLeft: timingBorder,
};

const timingHeadBackgroundSx = {
  backgroundImage: (theme: Theme) =>
    `linear-gradient(to bottom, ${theme.vars.palette.background.neutral}, ${theme.vars.palette.background.neutral})`,
};
