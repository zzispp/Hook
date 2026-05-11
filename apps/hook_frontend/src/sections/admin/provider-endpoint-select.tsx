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
  placeholder = '输入关键词搜索...',
  disabled,
  sx,
  onChange,
}: {
  value: string;
  options: SearchSelectOption[];
  label?: string;
  size?: 'small' | 'medium';
  placeholder?: string;
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
      noOptionsText="无匹配项"
      sx={sx}
      onChange={(_, option) => onChange(option?.value ?? '')}
      renderInput={(params) => <TextField {...params} label={label} placeholder={placeholder} />}
    />
  );
}
