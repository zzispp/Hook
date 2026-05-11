'use client';

import type { EditableConditionNode } from './provider-endpoint-rule-condition';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { createEmptyConditionLeaf } from './provider-endpoint-rule-condition';
import { ProviderEndpointConditionEditor } from './provider-endpoint-condition-editor';

export function RuleShell({
  marker,
  children,
  dragIndex,
  dragScope,
  condition,
  onMove,
  onRemove,
  onReorder,
  onConditionChange,
}: {
  marker: string;
  children: React.ReactNode;
  dragIndex: number;
  dragScope: string;
  condition: EditableConditionNode | null;
  onMove: (offset: -1 | 1) => void;
  onRemove: () => void;
  onReorder: (from: number, to: number) => void;
  onConditionChange: (condition: EditableConditionNode | null) => void;
}) {
  return (
    <Box sx={ruleShellSx} onDragOver={(event) => event.preventDefault()} onDrop={(event) => handleDrop(event, dragScope, dragIndex, onReorder)}>
      <Stack direction="row" spacing={0.75} alignItems="center" flexWrap="wrap" useFlexGap>
        <Tooltip title="调整顺序">
          <Box draggable sx={dragHandleSx} onDragStart={(event) => handleDragStart(event, dragScope, dragIndex)}>
            <Iconify icon="carbon:menu" width={16} />
          </Box>
        </Tooltip>
        <Typography variant="caption" sx={markerSx}>{marker}</Typography>
        {children}
        <IconButton size="small" title="上移" onClick={() => onMove(-1)}><Iconify icon="eva:arrow-upward-fill" width={14} /></IconButton>
        <IconButton size="small" title="下移" onClick={() => onMove(1)}><Iconify icon="eva:arrow-downward-fill" width={14} /></IconButton>
        <IconButton size="small" title="条件触发" color={condition ? 'primary' : 'default'} onClick={() => onConditionChange(condition ? null : createEmptyConditionLeaf())}>
          <Iconify icon="ic:round-filter-list" width={14} />
        </IconButton>
        <IconButton size="small" title="删除" onClick={onRemove}><Iconify icon="mingcute:close-line" width={14} /></IconButton>
      </Stack>
      {condition && (
        <Box sx={{ mt: 0.75 }}>
          <ProviderEndpointConditionEditor value={condition} onChange={onConditionChange} onRemove={() => onConditionChange(null)} />
        </Box>
      )}
    </Box>
  );
}

function handleDragStart(event: React.DragEvent<HTMLElement>, scope: string, index: number) {
  event.dataTransfer.effectAllowed = 'move';
  event.dataTransfer.setData('text/plain', `${scope}:${index}`);
}

function handleDrop(
  event: React.DragEvent<HTMLElement>,
  scope: string,
  target: number,
  onReorder: (from: number, to: number) => void
) {
  const [sourceScope, sourceIndex] = event.dataTransfer.getData('text/plain').split(':');
  const from = Number(sourceIndex);
  if (sourceScope !== scope || Number.isNaN(from) || from === target) return;
  onReorder(from, target);
}

const markerSx = { width: 12, fontWeight: 700, color: 'text.secondary' };
const dragHandleSx = { display: 'inline-flex', color: 'text.disabled', cursor: 'grab' };
const ruleShellSx = {
  px: 1,
  py: 0.75,
  borderRadius: 1,
  borderLeft: '4px solid',
  borderColor: 'text.disabled',
  bgcolor: 'action.hover',
};
