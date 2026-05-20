'use client';

import type { ModelsDevModelItem } from 'src/types/model';
import type { ProviderGroup } from './model-dev-picker-utils';

import { useState } from 'react';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import ListItem from '@mui/material/ListItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';

import {
  countModels,
  toggleProvider,
  providerLogoUrl,
  useProviderGroups,
} from './model-dev-picker-utils';

// ----------------------------------------------------------------------

type Props = {
  items: ModelsDevModelItem[];
  loading: boolean;
  error?: Error;
  query: string;
  selected?: ModelsDevModelItem | null;
  onQueryChange: (value: string) => void;
  onSelect: (item: ModelsDevModelItem) => void;
  onRetry: () => void;
};

export function ModelDevPicker({
  items,
  loading,
  error,
  query,
  selected,
  onQueryChange,
  onSelect,
  onRetry,
}: Props) {
  const { t } = useTranslate('admin');
  const groups = useProviderGroups(items, query);
  const modelCount = countModels(groups);
  const [expandedProvider, setExpandedProvider] = useState<string | null>(null);

  if (error) {
    return (
      <Alert
        severity="error"
        action={
          <Button color="inherit" size="small" onClick={onRetry}>
            {t('common.retry')}
          </Button>
        }
      >
        {error.message}
      </Alert>
    );
  }

  return (
    <Stack sx={{ minHeight: 0, borderRight: { md: 1 }, borderColor: 'divider', pr: { md: 2 } }}>
      <TextField
        fullWidth
        size="small"
        value={query}
        label={t('fields.searchModelDev')}
        onChange={(event) => onQueryChange(event.target.value)}
      />
      <Box sx={{ pt: 1.5, pb: 1 }}>
        <Typography variant="caption" color="text.secondary">
          {loading ? t('common.loading') : t('models.modelDevCount', { count: modelCount })}
        </Typography>
      </Box>
      <Divider />
      <Scrollbar sx={{ minHeight: 0, flex: 1 }}>
        <List dense disablePadding>
          {groups.map((group) => (
            <ProviderSection
              key={group.providerId}
              group={group}
              expanded={expandedProvider === group.providerId}
              selected={selected}
              onToggle={() => setExpandedProvider(toggleProvider(expandedProvider, group.providerId))}
              onSelect={onSelect}
            />
          ))}
        </List>
        {!loading && groups.length === 0 && (
          <Typography variant="body2" color="text.secondary" sx={{ py: 4, textAlign: 'center' }}>
            {t('common.noData')}
          </Typography>
        )}
      </Scrollbar>
    </Stack>
  );
}

function ProviderSection({
  group,
  expanded,
  selected,
  onToggle,
  onSelect,
}: {
  group: ProviderGroup;
  expanded: boolean;
  selected?: ModelsDevModelItem | null;
  onToggle: () => void;
  onSelect: (item: ModelsDevModelItem) => void;
}) {
  return (
    <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
      <ListItem disablePadding>
        <ListItemButton sx={{ minHeight: 40, px: 1, borderRadius: 1 }} onClick={onToggle}>
          <Iconify
            width={16}
            icon="eva:arrow-ios-forward-fill"
            sx={{
              mr: 1,
              color: 'text.secondary',
              transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
              transition: (theme) => theme.transitions.create('transform'),
            }}
          />
          <ProviderLogo providerId={group.providerId} providerName={group.providerName} />
          <ListItemText
            primary={group.providerName}
            primaryTypographyProps={{ variant: 'subtitle2', noWrap: true }}
            sx={{ minWidth: 0, ml: 1 }}
          />
          <Typography variant="caption" color="text.secondary">
            {group.models.length}
          </Typography>
        </ListItemButton>
      </ListItem>
      <Collapse in={expanded} timeout="auto" unmountOnExit>
        <Box sx={{ bgcolor: 'action.hover', py: 0.25 }}>
          {group.models.map((item) => (
            <ModelRow
              key={`${item.providerId}:${item.modelId}`}
              item={item}
              selected={isSelected(item, selected)}
              onSelect={() => onSelect(item)}
            />
          ))}
        </Box>
      </Collapse>
    </Box>
  );
}

function ProviderLogo({ providerId, providerName }: { providerId: string; providerName: string }) {
  return (
    <Box
      component="img"
      alt={providerName}
      src={providerLogoUrl(providerId)}
      onError={(event) => {
        event.currentTarget.style.display = 'none';
      }}
      sx={{ width: 18, height: 18, borderRadius: 0.5, flexShrink: 0 }}
    />
  );
}

function ModelRow({
  item,
  selected,
  onSelect,
}: {
  item: ModelsDevModelItem;
  selected: boolean;
  onSelect: () => void;
}) {
  return (
    <ListItem disablePadding>
      <ListItemButton
        selected={selected}
        sx={{ alignItems: 'flex-start', borderRadius: 1, my: 0.25, pl: 5, pr: 1 }}
        onClick={onSelect}
      >
        <ListItemText
          primary={item.modelName}
          secondary={
            <>
              <Typography component="span" variant="caption" color="text.secondary">
                {item.modelId}
              </Typography>
              <ModelBadges item={item} />
            </>
          }
          primaryTypographyProps={{ variant: 'subtitle2', noWrap: true }}
        />
      </ListItemButton>
    </ListItem>
  );
}

function ModelBadges({ item }: { item: ModelsDevModelItem }) {
  const { t } = useTranslate('admin');
  const badges = [
    item.official ? t('models.official') : undefined,
    item.supportsVision ? 'vision' : undefined,
    item.supportsToolCall ? 'tools' : undefined,
    item.supportsReasoning ? 'reasoning' : undefined,
  ].filter(Boolean);

  return (
    <Stack direction="row" flexWrap="wrap" sx={{ gap: 0.75, mt: 0.75 }}>
      {badges.map((badge) => (
        <Chip key={badge} size="small" label={badge} />
      ))}
      {item.deprecated && (
        <Chip
          size="small"
          color="warning"
          icon={<Iconify icon="solar:danger-triangle-bold" />}
          label={t('models.deprecated')}
        />
      )}
    </Stack>
  );
}

function isSelected(left: ModelsDevModelItem, right?: ModelsDevModelItem | null) {
  return left.providerId === right?.providerId && left.modelId === right.modelId;
}
