'use client';

import type {
  EditableBodyRule,
  EditableHeaderRule,
} from './provider-endpoint-rule-types';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/components/iconify';

import { emptyBodyRule, emptyHeaderRule } from './provider-endpoint-rule-types';
import { ProviderEndpointBodyRuleRow } from './provider-endpoint-body-rule-row';
import { ProviderEndpointHeaderRuleRow } from './provider-endpoint-header-rule-row';

export function ProviderEndpointRuleList({
  open,
  headerRules,
  bodyRules,
  onOpenChange,
  onHeaderRulesChange,
  onBodyRulesChange,
}: {
  open: boolean;
  headerRules: EditableHeaderRule[];
  bodyRules: EditableBodyRule[];
  onOpenChange: (open: boolean) => void;
  onHeaderRulesChange: (rules: EditableHeaderRule[]) => void;
  onBodyRulesChange: (rules: EditableBodyRule[]) => void;
}) {
  const total = effectiveHeaderRules(headerRules) + effectiveBodyRules(bodyRules);

  return (
    <Box>
      <RuleListHeader
        open={open}
        count={total}
        onToggle={() => onOpenChange(!open)}
        onAddHeader={() => onHeaderRulesChange([...headerRules, emptyHeaderRule()])}
        onAddBody={() => onBodyRulesChange([...bodyRules, emptyBodyRule('set')])}
      />
      <Collapse in={open}>
        <Stack spacing={1} sx={{ pt: 1.5 }}>
          <RuleHelp />
          {headerRules.map((rule, index) => (
            <ProviderEndpointHeaderRuleRow
              key={index}
              rule={rule}
              index={index}
              onChange={(next) => onHeaderRulesChange(replaceAt(headerRules, index, next))}
              onMove={(offset) => onHeaderRulesChange(moveAt(headerRules, index, index + offset))}
              onRemove={() => onHeaderRulesChange(removeAt(headerRules, index))}
              onReorder={(from, to) => onHeaderRulesChange(moveAt(headerRules, from, to))}
              onConditionChange={(condition) => onHeaderRulesChange(replaceAt(headerRules, index, { ...rule, condition }))}
            />
          ))}
          {bodyRules.map((rule, index) => (
            <ProviderEndpointBodyRuleRow
              key={index}
              rule={rule}
              index={index}
              onChange={(next) => onBodyRulesChange(replaceAt(bodyRules, index, next))}
              onMove={(offset) => onBodyRulesChange(moveAt(bodyRules, index, index + offset))}
              onRemove={() => onBodyRulesChange(removeAt(bodyRules, index))}
              onReorder={(from, to) => onBodyRulesChange(moveAt(bodyRules, from, to))}
              onConditionChange={(condition) => onBodyRulesChange(replaceAt(bodyRules, index, { ...rule, condition }))}
            />
          ))}
        </Stack>
      </Collapse>
    </Box>
  );
}

function RuleListHeader({
  open,
  count,
  onToggle,
  onAddHeader,
  onAddBody,
}: {
  open: boolean;
  count: number;
  onToggle: () => void;
  onAddHeader: () => void;
  onAddBody: () => void;
}) {
  return (
    <Stack direction="row" spacing={1} alignItems="center">
      <Button color="inherit" size="small" startIcon={<Iconify icon={open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />} onClick={onToggle}>
        请求规则
      </Button>
      {count > 0 && <Typography variant="caption" sx={countSx}>{count} 条</Typography>}
      <Box sx={{ flex: 1 }} />
      <Button size="small" startIcon={<Iconify icon="mingcute:add-line" />} onClick={onAddHeader}>请求头</Button>
      <Button size="small" startIcon={<Iconify icon="mingcute:add-line" />} onClick={onAddBody}>请求体</Button>
    </Stack>
  );
}

function RuleHelp() {
  return (
    <Stack spacing={0.5}>
      <Typography variant="caption" color="text.secondary">拖拽左侧手柄或使用上下按钮可调整规则执行顺序</Typography>
      <Typography variant="caption" color="text.secondary">
        <Box component="code" sx={codeSx}>.</Box> 嵌套字段 / <Box component="code" sx={codeSx}>[N]</Box> 数组索引 / <Box component="code" sx={codeSx}>[*]</Box> 通配符；值为 JSON 格式
      </Typography>
      <Divider />
    </Stack>
  );
}

function replaceAt<T>(items: T[], index: number, value: T) {
  return items.map((item, itemIndex) => (itemIndex === index ? value : item));
}

function removeAt<T>(items: T[], index: number) {
  return items.filter((_, itemIndex) => itemIndex !== index);
}

function moveAt<T>(items: T[], index: number, target: number) {
  if (target < 0 || target >= items.length) return items;
  const next = [...items];
  const [item] = next.splice(index, 1);
  next.splice(target, 0, item);
  return next;
}

function effectiveHeaderRules(rules: EditableHeaderRule[]) {
  return rules.filter((rule) => (rule.action === 'rename' ? rule.from.trim() && rule.to.trim() : rule.key.trim())).length;
}

function effectiveBodyRules(rules: EditableBodyRule[]) {
  return rules.filter((rule) => {
    if (rule.action === 'rename') return rule.from.trim() && rule.to.trim();
    if (rule.action === 'regex_replace') return rule.path.trim() && rule.pattern.trim();
    return rule.path.trim();
  }).length;
}

const countSx = { px: 1, py: 0.25, borderRadius: 999, bgcolor: 'action.selected', fontWeight: 700 };
const codeSx = { px: 0.5, borderRadius: 0.5, bgcolor: 'action.selected' };
