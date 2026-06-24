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

const TIMING_COLUMNS: readonly TimingColumn[] = [
  { metric: 'response_headers', labelKey: 'requestRecords.responseHeadersShort', width: 112 },
  { metric: 'first_sse_event', labelKey: 'requestRecords.firstSseEventShort', width: 112 },
  { metric: 'first_output', labelKey: 'requestRecords.firstOutputShort', width: 112 },
  { metric: 'total_latency', labelKey: 'requestRecords.totalLatencyShort', width: 120 },
];

export function requestRecordTimingHeadCells(t: (key: string) => string): TableHeadCellProps[] {
  return TIMING_COLUMNS.map((column) => ({
    id: column.metric,
    label: t(column.labelKey),
    width: column.width,
    sx: timingCellSx(column.metric, true),
  }));
}

export function RequestRecordTimingCells({
  record,
  now,
}: {
  record: RequestRecord;
  now: number;
}) {
  return TIMING_COLUMNS.map((column) => (
    <TableCell key={column.metric} sx={timingCellSx(column.metric, false)}>
      <RequestRecordDurationText record={record} metric={column.metric} now={now} />
    </TableCell>
  ));
}

function timingCellSx(metric: RequestTimingMetric, head: boolean): SxProps<Theme> {
  const column = timingColumn(metric);
  return {
    position: { xs: 'static', lg: 'sticky' },
    right: { lg: timingRightOffset(metric) },
    zIndex: { lg: head ? TIMING_HEAD_CELL_Z_INDEX : TIMING_CELL_Z_INDEX },
    width: column.width,
    minWidth: column.width,
    maxWidth: column.width,
    whiteSpace: 'nowrap',
    bgcolor: head ? 'background.neutral' : 'background.paper',
    ...(head ? timingHeadBackgroundSx : {}),
    ...(metric === TIMING_COLUMNS[0].metric ? timingDividerSx : {}),
  };
}

function timingRightOffset(metric: RequestTimingMetric) {
  const index = TIMING_COLUMNS.findIndex((column) => column.metric === metric);
  return TIMING_COLUMNS.slice(index + 1).reduce((sum, column) => sum + column.width, 0);
}

function timingColumn(metric: RequestTimingMetric) {
  return TIMING_COLUMNS.find((column) => column.metric === metric) ?? TIMING_COLUMNS[0];
}

const timingBorder = (theme: Theme) => `1px solid ${theme.vars.palette.divider}`;

const timingDividerSx = {
  borderLeft: timingBorder,
};

const timingHeadBackgroundSx = {
  backgroundImage: (theme: Theme) =>
    `linear-gradient(to bottom, ${theme.vars.palette.background.neutral}, ${theme.vars.palette.background.neutral})`,
};
