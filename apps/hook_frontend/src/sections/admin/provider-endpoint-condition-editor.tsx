'use client';

import type { BodyRuleConditionOp } from 'src/types/provider';
import type {
  ConditionSource,
  ConditionGroupMode,
  EditableConditionNode,
  EditableConditionLeaf,
  EditableConditionGroup,
} from './provider-endpoint-rule-condition';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import {
  CONDITION_OP_OPTIONS,
  createConditionGroup,
  cloneEditableCondition,
  createEmptyConditionLeaf,
  isConditionValueRequired,
  getConditionValuePlaceholder,
} from './provider-endpoint-rule-condition';

export function ProviderEndpointConditionEditor({
  value,
  nested = false,
  removable = false,
  pathHint = '字段路径',
  onChange,
  onRemove,
}: {
  value: EditableConditionNode;
  nested?: boolean;
  removable?: boolean;
  pathHint?: string;
  onChange: (value: EditableConditionNode) => void;
  onRemove?: () => void;
}) {
  if (value.kind === 'group') {
    return <ConditionGroup value={value} nested={nested} removable={removable} pathHint={pathHint} onChange={onChange} onRemove={onRemove} />;
  }
  return <ConditionLeaf value={value} nested={nested} removable={removable} pathHint={pathHint} onChange={onChange} onRemove={onRemove} />;
}

function ConditionGroup(props: {
  value: EditableConditionGroup;
  nested: boolean;
  removable: boolean;
  pathHint: string;
  onChange: (value: EditableConditionNode) => void;
  onRemove?: () => void;
}) {
  return (
    <Box sx={{ p: 1, borderRadius: 1, border: '1px dashed', borderColor: 'divider', bgcolor: props.nested ? 'background.paper' : 'action.hover' }}>
      <GroupHeader {...props} />
      <Stack spacing={0.5}>
        {props.value.children.map((child, index) => (
          <Box key={index}>
            {index > 0 && <LogicDivider mode={props.value.mode} />}
            <ProviderEndpointConditionEditor
              value={child}
              nested
              removable
              pathHint={props.pathHint}
              onChange={(next) => updateChild(props, index, next)}
              onRemove={() => removeChild(props, index)}
            />
          </Box>
        ))}
      </Stack>
    </Box>
  );
}

function GroupHeader(props: {
  value: EditableConditionGroup;
  nested: boolean;
  removable: boolean;
  onChange: (value: EditableConditionNode) => void;
  onRemove?: () => void;
}) {
  return (
    <Stack direction="row" spacing={0.75} alignItems="center" sx={{ mb: 1 }}>
      {!props.nested && <SmallIconButton title="转回单条件" icon="solar:list-bold" onClick={() => props.onChange(createEmptyConditionLeaf())} />}
      <SelectSmall value={props.value.mode} options={groupOptions} onChange={(mode) => updateGroupMode(props, mode as ConditionGroupMode)} />
      <Box sx={{ flex: 1 }} />
      <Button size="small" startIcon={<Iconify icon="mingcute:add-line" />} onClick={() => addLeafChild(props)}>
        条件
      </Button>
      {!props.nested && (
        <Button size="small" startIcon={<Iconify icon="mingcute:add-line" />} onClick={() => addGroupChild(props)}>
          子组
        </Button>
      )}
      {props.removable && props.nested && <SmallIconButton title="删除条件" icon="mingcute:close-line" onClick={props.onRemove} />}
    </Stack>
  );
}

function ConditionLeaf(props: {
  value: EditableConditionLeaf;
  nested: boolean;
  removable: boolean;
  pathHint: string;
  onChange: (value: EditableConditionNode) => void;
  onRemove?: () => void;
}) {
  return (
    <Stack direction="row" flexWrap="wrap" useFlexGap spacing={0.75} alignItems="center" sx={leafSx(props.nested)}>
      {!props.nested && <SmallIconButton title="转为组合条件 (AND/OR)" icon="solar:list-bold" onClick={() => props.onChange(createConditionGroup('all', [props.value]))} />}
      <SelectSmall value={props.value.source} options={sourceOptions} onChange={(source) => updateLeafField(props, 'source', source)} sx={{ width: 96 }} />
      <TextField size="small" value={props.value.path} placeholder={props.pathHint} onChange={(event) => updateLeafField(props, 'path', event.target.value)} sx={{ flex: 1, minWidth: 140 }} />
      <SelectSmall value={props.value.op} options={CONDITION_OP_OPTIONS} onChange={(op) => updateLeafField(props, 'op', op)} sx={{ width: 128 }} />
      {isConditionValueRequired(props.value.op) && (
        <TextField size="small" value={props.value.value} placeholder={getConditionValuePlaceholder(props.value.op)} onChange={(event) => updateLeafField(props, 'value', event.target.value)} sx={{ flex: 1, minWidth: 140 }} />
      )}
      {props.removable && <SmallIconButton title="删除条件" icon="mingcute:close-line" onClick={props.onRemove} />}
    </Stack>
  );
}

function SelectSmall({
  value,
  options,
  sx,
  onChange,
}: {
  value: string;
  options: Array<{ value: string; label: string }>;
  sx?: object;
  onChange: (value: string) => void;
}) {
  return (
    <TextField select size="small" value={value} onChange={(event) => onChange(event.target.value)} sx={sx}>
      {options.map((option) => <MenuItem key={option.value} value={option.value}>{option.label}</MenuItem>)}
    </TextField>
  );
}

function SmallIconButton({ title, icon, onClick }: { title: string; icon: React.ComponentProps<typeof Iconify>['icon']; onClick?: () => void }) {
  return (
    <IconButton size="small" title={title} onClick={onClick}>
      <Iconify icon={icon} width={16} />
    </IconButton>
  );
}

function LogicDivider({ mode }: { mode: ConditionGroupMode }) {
  return (
    <Stack direction="row" spacing={1} alignItems="center" sx={{ py: 0.5, pl: 1 }}>
      <Typography variant="caption" sx={{ px: 0.75, borderRadius: 0.75, bgcolor: mode === 'all' ? 'info.lighter' : 'warning.lighter', color: mode === 'all' ? 'info.dark' : 'warning.dark', fontWeight: 700 }}>
        {mode === 'all' ? 'AND' : 'OR'}
      </Typography>
      <Box sx={{ flex: 1, borderTop: '1px dashed', borderColor: 'divider' }} />
    </Stack>
  );
}

function updateLeafField(props: { value: EditableConditionLeaf; onChange: (value: EditableConditionNode) => void }, field: keyof EditableConditionLeaf, raw: string) {
  const next = { ...props.value };
  if (field === 'op') {
    next.op = raw as BodyRuleConditionOp;
    if (!isConditionValueRequired(next.op)) next.value = '';
  } else if (field === 'source') {
    next.source = raw as ConditionSource;
  } else if (field === 'path' || field === 'value') {
    next[field] = raw;
  }
  props.onChange(next);
}

function updateGroupMode(props: { value: EditableConditionGroup; onChange: (value: EditableConditionNode) => void }, mode: ConditionGroupMode) {
  props.onChange({ ...props.value, mode });
}

function addLeafChild(props: { value: EditableConditionGroup; onChange: (value: EditableConditionNode) => void }) {
  props.onChange({ ...props.value, children: [...props.value.children, createEmptyConditionLeaf()] });
}

function addGroupChild(props: { value: EditableConditionGroup; onChange: (value: EditableConditionNode) => void }) {
  const mode = props.value.mode === 'all' ? 'any' : 'all';
  props.onChange({ ...props.value, children: [...props.value.children, createConditionGroup(mode)] });
}

function updateChild(props: { value: EditableConditionGroup; onChange: (value: EditableConditionNode) => void }, index: number, child: EditableConditionNode) {
  const children = props.value.children.map((item, itemIndex) => (itemIndex === index ? child : item));
  props.onChange({ ...props.value, children });
}

function removeChild(props: { value: EditableConditionGroup; onChange: (value: EditableConditionNode) => void }, index: number) {
  props.onChange({ ...props.value, children: props.value.children.filter((_, itemIndex) => itemIndex !== index).map(cloneEditableCondition) });
}

const groupOptions = [
  { value: 'all', label: 'AND' },
  { value: 'any', label: 'OR' },
];

const sourceOptions = [
  { value: 'current', label: 'Current' },
  { value: 'original', label: 'Original' },
];

function leafSx(nested: boolean) {
  return { p: 0.75, borderRadius: 1, border: nested ? 0 : '1px solid', borderColor: 'divider', bgcolor: nested ? 'background.paper' : 'action.hover' };
}
