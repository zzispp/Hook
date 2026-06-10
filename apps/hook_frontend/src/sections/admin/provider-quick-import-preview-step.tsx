'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { GlobalModelResponse } from 'src/types/model';
import type {
  ProviderQuickImportTokenPreview,
  ProviderQuickImportPreviewResponse,
} from 'src/types/provider-quick-import';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';

import { useTranslate } from 'src/locales/use-locales';

import { type QuickImportTokenDraft } from './provider-quick-import-utils';
import { ProviderQuickImportTokenRow } from './provider-quick-import-token-row';

const CHECKBOX_COLUMN_WIDTH = 56;
const KEY_COLUMN_WIDTH = '38%';
const STATUS_COLUMN_WIDTH = 96;
const UPSTREAM_RATIO_COLUMN_WIDTH = 180;
const RECHARGE_MULTIPLIER_COLUMN_WIDTH = 120;

type PreviewStepProps = {
  preview: ProviderQuickImportPreviewResponse;
  models: GlobalModelResponse[];
  tokens: Record<string, QuickImportTokenDraft>;
  mappings: Record<string, string>;
  setTokens: Dispatch<SetStateAction<Record<string, QuickImportTokenDraft>>>;
  setMappings: Dispatch<SetStateAction<Record<string, string>>>;
  onMapModels: (token: ProviderQuickImportTokenPreview) => void;
};

export function ProviderQuickImportPreviewStep(props: PreviewStepProps) {
  return (
    <Stack spacing={2}>
      <TokenTable {...props} />
    </Stack>
  );
}

function TokenTable({ preview, tokens, mappings, setTokens, onMapModels }: PreviewStepProps) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ width: '100%', maxWidth: '100%', pb: 1 }}>
      <Table size="small" sx={{ width: '100%', tableLayout: 'fixed' }}>
        <TableHead>
          <TableRow>
            <TableCell padding="checkbox" sx={{ width: CHECKBOX_COLUMN_WIDTH, minWidth: CHECKBOX_COLUMN_WIDTH }} />
            <TableCell sx={{ width: KEY_COLUMN_WIDTH, minWidth: KEY_COLUMN_WIDTH }}>{t('fields.keyName')}</TableCell>
            <TableCell sx={{ width: STATUS_COLUMN_WIDTH, minWidth: STATUS_COLUMN_WIDTH }}>{t('common.status')}</TableCell>
            <TableCell sx={{ width: UPSTREAM_RATIO_COLUMN_WIDTH, minWidth: UPSTREAM_RATIO_COLUMN_WIDTH }}>
              {t('providers.quickImportUpstreamRatio')}
            </TableCell>
            <TableCell sx={{ width: RECHARGE_MULTIPLIER_COLUMN_WIDTH, minWidth: RECHARGE_MULTIPLIER_COLUMN_WIDTH }}>
              {t('providers.quickImportRechargeMultiplier')}
            </TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {preview.tokens.map((token) => (
            <ProviderQuickImportTokenRow
              key={token.upstream_token_id}
              token={token}
              draft={tokens[token.upstream_token_id]}
              mappings={mappings}
              rechargeMultiplier={preview.recharge_multiplier}
              setTokens={setTokens}
              onMapModels={onMapModels}
            />
          ))}
        </TableBody>
      </Table>
    </Box>
  );
}
