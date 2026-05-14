'use client';

import type { TFunction } from 'i18next';
import type { TextFieldProps } from '@mui/material/TextField';
import type { CardCodeType } from 'src/types/card-code';
import type { CardCodeFilters, CardCodeTypeFilters } from 'src/actions/card-code';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

import { ALL_FILTER_VALUE } from 'src/sections/wallet/wallet-constants';

const EMPTY_STATUS_FILTER = '';
const TOOLBAR_BUTTON_SX = {
  flexShrink: 0,
  minWidth: 112,
  whiteSpace: 'nowrap',
  width: { xs: '100%', md: 'auto' },
};

export type CardCodeFilterState = {
  search: string;
  status: string;
  typeId: string;
};

export type CardCodeTypeFilterState = {
  search: string;
  status: string;
};

export const DEFAULT_CARD_CODE_FILTERS: CardCodeFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
  typeId: ALL_FILTER_VALUE,
};

export const DEFAULT_CARD_CODE_TYPE_FILTERS: CardCodeTypeFilterState = {
  search: '',
  status: EMPTY_STATUS_FILTER,
};

type CodeToolbarProps = {
  t: TFunction<'admin'>;
  filters: CardCodeFilterState;
  types: CardCodeType[];
  selectedCount: number;
  busy: boolean;
  onChange: (filters: CardCodeFilterState) => void;
  onGenerate: VoidFunction;
  onExportCsv: VoidFunction;
  onExportTxt: VoidFunction;
  onEnable: VoidFunction;
  onDisable: VoidFunction;
};

export function CardCodeToolbar({ t, filters, types, selectedCount, busy, onChange, ...actions }: CodeToolbarProps) {
  const patchFilters = (patch: Partial<CardCodeFilterState>) => onChange({ ...filters, ...patch });

  return (
    <Stack spacing={2} sx={{ p: 2.5 }}>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <SearchField value={filters.search} placeholder={t('adminCardCodes.filters.searchCodes')} onChange={(search) => patchFilters({ search })} />
        <TextField
          select
          label={t('common.status')}
          value={filters.status}
          sx={{ minWidth: 160 }}
          InputLabelProps={{ shrink: true }}
          SelectProps={statusSelectProps(t)}
          onChange={(event) => patchFilters({ status: event.target.value })}
        >
          <StatusOptions t={t} includeUsed />
        </TextField>
        <TextField select label={t('adminCardCodes.fields.type')} value={filters.typeId} sx={{ minWidth: 180 }} onChange={(event) => patchFilters({ typeId: event.target.value })}>
          <MenuItem value={ALL_FILTER_VALUE}>{t('adminCardCodes.filters.allTypes')}</MenuItem>
          {types.map((type) => (
            <MenuItem key={type.id} value={type.id}>
              {type.name}
            </MenuItem>
          ))}
        </TextField>
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5} justifyContent="flex-end">
        <Button variant="outlined" disabled={busy} startIcon={<Iconify icon="solar:download-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={actions.onExportCsv}>
          {selectedCount > 0 ? t('adminCardCodes.actions.exportSelectedCsv') : t('adminCardCodes.actions.exportCsv')}
        </Button>
        <Button variant="outlined" disabled={busy} startIcon={<Iconify icon="solar:file-text-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={actions.onExportTxt}>
          {selectedCount > 0 ? t('adminCardCodes.actions.exportSelectedTxt') : t('adminCardCodes.actions.exportTxt')}
        </Button>
        <Button variant="outlined" disabled={busy || selectedCount === 0} startIcon={<Iconify icon="solar:check-circle-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={actions.onEnable}>
          {t('adminCardCodes.actions.enable')}
        </Button>
        <Button variant="outlined" color="warning" disabled={busy || selectedCount === 0} startIcon={<Iconify icon="solar:stop-circle-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={actions.onDisable}>
          {t('adminCardCodes.actions.disable')}
        </Button>
        <Button variant="contained" disabled={busy} startIcon={<Iconify icon="solar:add-circle-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={actions.onGenerate}>
          {t('adminCardCodes.actions.generate')}
        </Button>
      </Stack>
    </Stack>
  );
}

export function CardCodeTypeToolbar({
  t,
  filters,
  busy,
  onChange,
  onCreate,
}: {
  t: TFunction<'admin'>;
  filters: CardCodeTypeFilterState;
  busy: boolean;
  onChange: (filters: CardCodeTypeFilterState) => void;
  onCreate: VoidFunction;
}) {
  const patchFilters = (patch: Partial<CardCodeTypeFilterState>) => onChange({ ...filters, ...patch });

  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2.5 }}>
      <SearchField value={filters.search} placeholder={t('adminCardCodes.filters.searchTypes')} onChange={(search) => patchFilters({ search })} />
      <TextField
        select
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 160 }}
        InputLabelProps={{ shrink: true }}
        SelectProps={statusSelectProps(t)}
        onChange={(event) => patchFilters({ status: event.target.value })}
      >
        <StatusOptions t={t} />
      </TextField>
      <Button variant="contained" disabled={busy} startIcon={<Iconify icon="solar:add-circle-bold" />} sx={TOOLBAR_BUTTON_SX} onClick={onCreate}>
        {t('adminCardCodes.actions.createType')}
      </Button>
    </Stack>
  );
}

export function toCardCodeFilters(filters: CardCodeFilterState): CardCodeFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
    type_id: filters.typeId === ALL_FILTER_VALUE ? undefined : filters.typeId,
  };
}

export function toCardCodeTypeFilters(filters: CardCodeTypeFilterState): CardCodeTypeFilters {
  return {
    search: filters.search.trim() || undefined,
    status: filters.status || undefined,
  };
}

function SearchField({ value, placeholder, onChange }: { value: string; placeholder: string; onChange: (value: string) => void }) {
  return (
    <TextField
      fullWidth
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      slotProps={{ input: { startAdornment: <InputAdornment position="start"><Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} /></InputAdornment> } }}
    />
  );
}

function StatusOptions({ t, includeUsed = false }: { t: TFunction<'admin'>; includeUsed?: boolean }) {
  return (
    <>
      <MenuItem value={EMPTY_STATUS_FILTER}>{t('filters.allStatuses')}</MenuItem>
      <MenuItem value="active">{t('adminCardCodes.status.active')}</MenuItem>
      <MenuItem value="disabled">{t('adminCardCodes.status.disabled')}</MenuItem>
      {includeUsed ? <MenuItem value="used">{t('adminCardCodes.status.used')}</MenuItem> : null}
      {includeUsed ? <MenuItem value="expired">{t('adminCardCodes.status.expired')}</MenuItem> : null}
    </>
  );
}

function statusSelectProps(t: TFunction<'admin'>): TextFieldProps['SelectProps'] {
  return {
    displayEmpty: true,
    renderValue: (selected) => statusFilterLabel(t, String(selected)),
  };
}

function statusFilterLabel(t: TFunction<'admin'>, status: string) {
  return status ? t(`adminCardCodes.status.${status}`) : t('filters.allStatuses');
}
