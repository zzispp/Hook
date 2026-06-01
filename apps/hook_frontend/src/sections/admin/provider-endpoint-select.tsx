'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import TextField from '@mui/material/TextField';
import Autocomplete from '@mui/material/Autocomplete';

export type SearchSelectOption = {
  value: string;
  label: string;
};

export function ProviderEndpointSearchSelect({
  value,
  options,
  label,
  size = 'small',
  placeholder,
  noOptionsText,
  disabled,
  sx,
  onChange,
}: {
  value: string;
  options: SearchSelectOption[];
  label?: string;
  size?: 'small' | 'medium';
  placeholder?: string;
  noOptionsText?: string;
  disabled?: boolean;
  sx?: SxProps<Theme>;
  onChange: (value: string) => void;
}) {
  const selected = options.find((option) => option.value === value) ?? null;

  return (
    <Autocomplete
      disableClearable={false}
      disabled={disabled}
      options={options}
      size={size}
      value={selected}
      getOptionLabel={(option) => option.label}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      noOptionsText={noOptionsText}
      sx={sx}
      onChange={(_, option) => onChange(option?.value ?? '')}
      renderInput={(params) => <TextField {...params} label={label} placeholder={placeholder} />}
    />
  );
}

export function ProviderEndpointMultiSearchSelect({
  value,
  options,
  label,
  size = 'small',
  placeholder,
  noOptionsText,
  disabled,
  sx,
  onChange,
}: {
  value: string[];
  options: SearchSelectOption[];
  label?: string;
  size?: 'small' | 'medium';
  placeholder?: string;
  noOptionsText?: string;
  disabled?: boolean;
  sx?: SxProps<Theme>;
  onChange: (value: string[]) => void;
}) {
  const selected = options.filter((option) => value.includes(option.value));

  return (
    <Autocomplete
      multiple
      disabled={disabled}
      options={options}
      size={size}
      value={selected}
      getOptionLabel={(option) => option.label}
      isOptionEqualToValue={(option, current) => option.value === current.value}
      noOptionsText={noOptionsText}
      sx={sx}
      onChange={(_, items) => onChange(items.map((item) => item.value))}
      renderInput={(params) => <TextField {...params} label={label} placeholder={placeholder} />}
    />
  );
}
