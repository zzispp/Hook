'use client';

import type { EditableConditionNode } from './provider-endpoint-rule-condition';
import type { EditableBodyRule, EditableBodyRuleAction } from './provider-endpoint-rule-types';

import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/components/iconify';

import { RuleShell } from './provider-endpoint-rule-shell';
import { ProviderEndpointSearchSelect } from './provider-endpoint-select';
import {
  emptyBodyRule,
  getRegexValidation,
  BODY_ACTION_OPTIONS,
  getBodyJsonValidation,
  BODY_NAME_STYLE_OPTIONS,
} from './provider-endpoint-rule-types';

export function ProviderEndpointBodyRuleRow({
  rule,
  index,
  onMove,
  onChange,
  onRemove,
  onReorder,
  onConditionChange,
}: {
  rule: EditableBodyRule;
  index: number;
  onMove: (offset: -1 | 1) => void;
  onChange: (rule: EditableBodyRule) => void;
  onRemove: () => void;
  onReorder: (from: number, to: number) => void;
  onConditionChange: (condition: EditableConditionNode | null) => void;
}) {
  return (
    <RuleShell marker="B" dragScope="body" dragIndex={index} condition={rule.condition} onMove={onMove} onRemove={onRemove} onReorder={onReorder} onConditionChange={onConditionChange}>
      <ProviderEndpointSearchSelect
        value={rule.action}
        options={BODY_ACTION_OPTIONS}
        sx={{ width: 128 }}
        onChange={(action) => onChange({ ...emptyBodyRule(action as EditableBodyRuleAction), condition: rule.condition })}
      />
      <BodyRuleFields rule={rule} onChange={onChange} />
    </RuleShell>
  );
}

function BodyRuleFields({
  rule,
  onChange,
}: {
  rule: EditableBodyRule;
  onChange: (rule: EditableBodyRule) => void;
}) {
  if (rule.action === 'rename') return <RenameFields rule={rule} onChange={onChange} />;
  if (rule.action === 'insert') return <InsertFields rule={rule} onChange={onChange} />;
  if (rule.action === 'regex_replace') return <RegexFields rule={rule} onChange={onChange} />;
  if (rule.action === 'name_style') return <NameStyleFields rule={rule} onChange={onChange} />;
  return <PathValueFields rule={rule} onChange={onChange} />;
}

function PathValueFields({ rule, onChange }: RowProps) {
  return (
    <>
      <CompactField value={rule.path} placeholder={rule.action === 'drop' ? '要删除的字段路径' : '字段路径（如 metadata.user_id）'} onChange={(path) => onChange({ ...rule, path })} />
      {rule.action === 'set' && <JsonValueField rule={rule} onChange={onChange} />}
    </>
  );
}

function RenameFields({ rule, onChange }: RowProps) {
  return (
    <>
      <CompactField value={rule.from} placeholder="来源字段路径" onChange={(from) => onChange({ ...rule, from })} />
      <Box component="span" sx={{ color: 'text.secondary' }}>→</Box>
      <CompactField value={rule.to} placeholder="目标字段路径" onChange={(to) => onChange({ ...rule, to })} />
    </>
  );
}

function InsertFields({ rule, onChange }: RowProps) {
  return (
    <>
      <CompactField value={rule.path} placeholder="目标数组路径" onChange={(path) => onChange({ ...rule, path })} />
      <CompactField value={rule.index} placeholder="位置，留空追加" onChange={(index) => onChange({ ...rule, index })} sx={{ maxWidth: 140 }} />
      <JsonValueField rule={rule} onChange={onChange} />
    </>
  );
}

function RegexFields({ rule, onChange }: RowProps) {
  return (
    <>
      <CompactField value={rule.path} placeholder="字符串字段路径" onChange={(path) => onChange({ ...rule, path })} />
      <CompactField value={rule.pattern} placeholder="正则表达式" onChange={(pattern) => onChange({ ...rule, pattern })} />
      <CompactField value={rule.replacement} placeholder="替换为" onChange={(replacement) => onChange({ ...rule, replacement })} />
      <CompactField value={rule.flags} placeholder="flags: i/m/s" onChange={(flags) => onChange({ ...rule, flags })} sx={{ maxWidth: 120 }} />
      <ValidationIcon valid={getRegexValidation(rule)} />
    </>
  );
}

function NameStyleFields({ rule, onChange }: RowProps) {
  return (
    <>
      <CompactField value={rule.path} placeholder="字段路径" onChange={(path) => onChange({ ...rule, path })} />
      <ProviderEndpointSearchSelect value={rule.style} options={BODY_NAME_STYLE_OPTIONS} sx={{ width: 160 }} onChange={(style) => onChange({ ...rule, style })} />
    </>
  );
}

function JsonValueField({ rule, onChange }: RowProps) {
  return (
    <>
      <Box component="span" sx={{ color: 'text.secondary' }}>=</Box>
      <CompactField value={rule.value} placeholder='123 / "text" / {{$original}}' onChange={(value) => onChange({ ...rule, value })} />
      <ValidationIcon valid={getBodyJsonValidation(rule)} />
    </>
  );
}

function ValidationIcon({ valid }: { valid: boolean | null }) {
  if (valid === null) return null;
  return (
    <IconButton size="small" color={valid ? 'success' : 'error'} tabIndex={-1}>
      <Iconify icon={valid ? 'solar:check-circle-bold' : 'solar:close-circle-bold'} width={16} />
    </IconButton>
  );
}

function CompactField({
  value,
  placeholder,
  sx,
  onChange,
}: {
  value: string;
  placeholder: string;
  sx?: object;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      size="small"
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      sx={{ flex: 1, minWidth: 140, ...sx }}
    />
  );
}

type RowProps = {
  rule: EditableBodyRule;
  onChange: (rule: EditableBodyRule) => void;
};
