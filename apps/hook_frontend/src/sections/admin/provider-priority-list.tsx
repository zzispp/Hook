'use client';

import type { PriorityItem } from './provider-priority-utils';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

type PriorityListProps = {
  items: PriorityItem[];
  loading: boolean;
  editingId: string | null;
  draggingId: string | null;
  onDragStart: (id: string) => void;
  onDragEnd: () => void;
  onDrop: (id: string) => void;
  onEdit: (id: string | null) => void;
  onPriorityChange: (id: string, value: string) => void;
};

export function ProviderPriorityList(props: PriorityListProps) {
  const { t } = useTranslate('admin');

  if (props.loading) return <LoadingState />;
  if (props.items.length === 0) return <EmptyState />;

  return (
    <Stack sx={{ height: 'min(65vh, 520px)', overflowY: 'auto', pr: 0.5, gap: 0.75 }}>
      {props.items.map((item) => (
        <PriorityRow key={item.id} item={item} {...props} />
      ))}
      <Typography variant="caption" color="text.secondary" sx={{ pt: 0.5 }}>
        {t('providers.samePriorityHint')}
      </Typography>
    </Stack>
  );
}

function PriorityRow({
  item,
  draggingId,
  editingId,
  onDragStart,
  onDragEnd,
  onDrop,
  onEdit,
  onPriorityChange,
}: PriorityListProps & { item: PriorityItem }) {
  const dragging = draggingId === item.id;

  return (
    <Box
      draggable
      onDragStart={() => onDragStart(item.id)}
      onDragEnd={onDragEnd}
      onDragOver={(event) => event.preventDefault()}
      onDrop={() => onDrop(item.id)}
      sx={rowSx(dragging)}
    >
      <Iconify icon="custom:drag-dots-fill" width={20} sx={{ color: 'text.disabled', cursor: 'grab' }} />
      <PriorityValue
        item={item}
        editing={editingId === item.id}
        onEdit={onEdit}
        onPriorityChange={onPriorityChange}
      />
      <Box sx={{ minWidth: 0, flex: 1 }}>
        <Typography variant="subtitle2" noWrap>
          {item.name}
        </Typography>
        {!item.is_active && (
          <Stack direction="row" spacing={0.75} sx={{ mt: 0.5 }}>
            <DisabledChip />
          </Stack>
        )}
        {(item.providerName || item.apiFormats?.length) && (
          <Stack direction="row" spacing={0.75} sx={{ mt: 0.5 }} useFlexGap flexWrap="wrap">
            {item.providerName ? <Chip size="small" variant="soft" label={item.providerName} /> : null}
            {item.apiFormats?.map((format) => (
              <Chip key={format} size="small" label={format} />
            ))}
          </Stack>
        )}
      </Box>
    </Box>
  );
}

function DisabledChip() {
  const { t } = useTranslate('admin');

  return <Chip size="small" label={t('common.disabled')} color="default" />;
}

function PriorityValue({
  item,
  editing,
  onEdit,
  onPriorityChange,
}: {
  item: PriorityItem;
  editing: boolean;
  onEdit: (id: string | null) => void;
  onPriorityChange: (id: string, value: string) => void;
}) {
  const { t } = useTranslate('admin');

  if (editing) {
    return (
      <TextField
        autoFocus
        size="small"
        type="number"
        value={item.priorityText}
        onBlur={() => onEdit(null)}
        onChange={(event) => onPriorityChange(item.id, event.target.value)}
        slotProps={{ htmlInput: { step: 1 } }}
        sx={{ width: 64 }}
      />
    );
  }

  return (
    <Tooltip title={t('providers.samePriorityHint')}>
      <Button color="inherit" variant="soft" size="small" onClick={() => onEdit(item.id)} sx={{ minWidth: 44 }}>
        {item.priorityText}
      </Button>
    </Tooltip>
  );
}

function LoadingState() {
  const { t } = useTranslate('admin');

  return (
    <Stack alignItems="center" justifyContent="center" sx={{ height: 280 }} spacing={1}>
      <CircularProgress size={24} />
      <Typography variant="body2" color="text.secondary">
        {t('common.loading')}
      </Typography>
    </Stack>
  );
}

function EmptyState() {
  const { t } = useTranslate('admin');

  return (
    <Stack alignItems="center" justifyContent="center" sx={{ height: 280 }}>
      <Typography variant="body2" color="text.secondary">
        {t('providers.noProviders')}
      </Typography>
    </Stack>
  );
}

function rowSx(dragging: boolean) {
  return {
    display: 'flex',
    alignItems: 'center',
    gap: 1.5,
    px: 1.5,
    py: 1,
    border: '1px solid',
    borderColor: 'divider',
    borderRadius: 1,
    bgcolor: dragging ? 'action.hover' : 'background.paper',
    opacity: dragging ? 0.72 : 1,
    transition: 'background-color 150ms ease, opacity 150ms ease',
  };
}
