'use client';

import type { GlobalModelResponse } from 'src/types/model';
import type { CurrencyDisplay } from 'src/utils/currency-format';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/locales/use-locales';

import { Label } from 'src/components/label';

import { ModelCopyButton } from './model-copy-button';
import { priceSummary, formatUsageCount } from './model-catalog-utils';

// ----------------------------------------------------------------------

type Props = {
  rows: GlobalModelResponse[];
  currencyDisplay?: CurrencyDisplay;
  onSelectRow: (row: GlobalModelResponse) => void;
};

export function ModelCatalogCards({ rows, currencyDisplay, onSelectRow }: Props) {
  return (
    <Stack spacing={2}>
      {rows.map((row) => (
        <CatalogCard
          key={row.id}
          row={row}
          currencyDisplay={currencyDisplay}
          onSelectRow={onSelectRow}
        />
      ))}
    </Stack>
  );
}

function CatalogCard({
  row,
  currencyDisplay,
  onSelectRow,
}: {
  row: GlobalModelResponse;
  currencyDisplay?: CurrencyDisplay;
  onSelectRow: (row: GlobalModelResponse) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Paper
      variant="outlined"
      onClick={() => onSelectRow(row)}
      sx={{ p: 2, borderRadius: 1, cursor: 'pointer' }}
    >
      <Stack spacing={2}>
        <Stack direction="row" alignItems="flex-start" justifyContent="space-between" spacing={2}>
          <Box sx={{ minWidth: 0 }}>
            <Typography variant="subtitle2" noWrap>
              {row.display_name}
            </Typography>
            <Stack direction="row" alignItems="center" spacing={0.5} sx={{ minWidth: 0 }}>
              <Typography variant="caption" color="text.secondary" sx={{ fontFamily: 'monospace' }}>
                {row.name}
              </Typography>
              <ModelCopyButton value={row.name} />
            </Stack>
          </Box>
          <Label color={row.is_active ? 'success' : 'default'} variant="soft">
            {row.is_active ? t('models.available') : t('models.unavailable')}
          </Label>
        </Stack>

        <Stack direction="row" flexWrap="wrap" sx={{ gap: 1 }}>
          <Chip
            size="small"
            label={`${t('models.pricing')}: ${priceSummary(row, currencyDisplay)}`}
          />
          <Chip
            size="small"
            label={`${t('models.usageCount')}: ${formatUsageCount(row.usage_count)}`}
          />
        </Stack>
      </Stack>
    </Paper>
  );
}
