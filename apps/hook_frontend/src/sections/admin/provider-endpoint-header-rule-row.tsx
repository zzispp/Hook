'use client';

import type { EditableConditionNode } from './provider-endpoint-rule-condition';
import type { HeaderRuleAction, EditableHeaderRule } from './provider-endpoint-rule-types';

import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';

import { RuleShell } from './provider-endpoint-rule-shell';
import { ProviderEndpointSearchSelect } from './provider-endpoint-select';
import { emptyHeaderRule, HEADER_ACTION_OPTIONS } from './provider-endpoint-rule-types';

export function ProviderEndpointHeaderRuleRow({
  rule,
  index,
  onMove,
  onChange,
  onRemove,
  onReorder,
  onConditionChange,
}: {
  rule: EditableHeaderRule;
  index: number;
  onMove: (offset: -1 | 1) => void;
  onChange: (rule: EditableHeaderRule) => void;
  onRemove: () => void;
  onReorder: (from: number, to: number) => void;
  onConditionChange: (condition: EditableConditionNode | null) => void;
}) {
  return (
    <RuleShell marker="H" dragScope="header" dragIndex={index} condition={rule.condition} onMove={onMove} onRemove={onRemove} onReorder={onReorder} onConditionChange={onConditionChange}>
      <ProviderEndpointSearchSelect
        value={rule.action}
        options={HEADER_ACTION_OPTIONS}
        sx={{ width: 112 }}
        onChange={(action) => onChange({ ...emptyHeaderRule(action as HeaderRuleAction), condition: rule.condition })}
      />
      <HeaderRuleFields rule={rule} onChange={onChange} />
    </RuleShell>
  );
}

function HeaderRuleFields({
  rule,
  onChange,
}: {
  rule: EditableHeaderRule;
  onChange: (rule: EditableHeaderRule) => void;
}) {
  if (rule.action === 'rename') {
    return (
      <>
        <CompactField value={rule.from} placeholder="来源请求头" onChange={(from) => onChange({ ...rule, from })} />
        <Box component="span" sx={{ color: 'text.secondary' }}>→</Box>
        <CompactField value={rule.to} placeholder="目标请求头" onChange={(to) => onChange({ ...rule, to })} />
      </>
    );
  }

  return (
    <>
      <CompactField value={rule.key} placeholder={rule.action === 'drop' ? '要删除的请求头' : '请求头名称'} onChange={(key) => onChange({ ...rule, key })} />
      {rule.action === 'set' && (
        <>
          <Box component="span" sx={{ color: 'text.secondary' }}>=</Box>
          <CompactField value={rule.value} placeholder="请求头值" onChange={(value) => onChange({ ...rule, value })} />
        </>
      )}
    </>
  );
}

function CompactField({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <TextField
      size="small"
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      sx={{ flex: 1, minWidth: 140 }}
    />
  );
}
