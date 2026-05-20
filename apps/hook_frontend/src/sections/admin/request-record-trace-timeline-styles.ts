import type { Theme } from '@mui/material/styles';
import type { TraceStatus } from './request-record-trace-timeline-utils';

import { statusColor } from './request-record-trace-timeline-utils';

export const panelSx = {
  p: 2,
  borderRadius: 1,
  border: (theme: Theme) => `1px solid ${theme.vars.palette.divider}`,
};

export const trackSx = { gap: 0, px: 2, py: 4, overflowX: 'auto', overflowY: 'visible' };
export const nodeWrapSx = { position: 'relative', minWidth: 128, px: 1 };
export const nodeLabelSx = { position: 'absolute', bottom: 'calc(100% + 8px)', maxWidth: 90 };
export const keyDotsSx = {
  position: 'absolute',
  top: 'calc(100% + 8px)',
  left: '50%',
  px: 0.5,
  py: 0.5,
  width: 'max-content',
  overflow: 'visible',
  transform: 'translateX(-50%)',
};
export const hiddenSummarySx = {
  position: 'absolute',
  top: 'calc(100% + 28px)',
  left: '50%',
  color: 'text.secondary',
  transform: 'translateX(-50%)',
};
export const lineSx = { width: 64, height: 2, bgcolor: 'divider' };
export const detailSx = { p: 1.5, borderRadius: 1, bgcolor: 'background.neutral' };

export function providerDotSx(status: TraceStatus, selected: boolean) {
  return {
    m: 0,
    width: 16,
    height: 16,
    p: 0,
    display: 'inline-flex',
    position: 'relative',
    alignItems: 'center',
    justifyContent: 'center',
    appearance: 'none',
    borderRadius: '50%',
    color: statusColor(status),
    border: '2px solid currentColor',
    bgcolor: 'transparent',
    cursor: 'pointer',
    overflow: 'visible',
    transform: selected ? 'scale(1.18)' : 'scale(1)',
    '&::before': dotBeforeSx(8),
  };
}

export function keyDotSx(status: TraceStatus, selected: boolean) {
  return {
    width: 10,
    height: 10,
    p: 0,
    border: 0,
    borderRadius: '50%',
    bgcolor: statusColor(status),
    cursor: 'pointer',
    opacity: selected ? 1 : 0.55,
    boxShadow: selected ? `0 0 0 2px #fff, 0 0 0 3px ${statusColor(status)}` : 'none',
  };
}

export function titleDotSx(status: TraceStatus) {
  return { width: 10, height: 10, borderRadius: '50%', bgcolor: statusColor(status), flexShrink: 0 };
}

function dotBeforeSx(size: number) {
  return {
    content: '""',
    position: 'absolute',
    top: '50%',
    left: '50%',
    width: size,
    height: size,
    borderRadius: '50%',
    bgcolor: 'currentColor',
    transform: 'translate(-50%, -50%)',
  };
}
