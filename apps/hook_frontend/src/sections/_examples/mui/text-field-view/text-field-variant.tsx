import type { TextFieldProps } from '@mui/material/TextField';

import { useState, useCallback } from 'react';

import Input from '@mui/material/Input';
import Select from '@mui/material/Select';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import InputLabel from '@mui/material/InputLabel';
import FormControl from '@mui/material/FormControl';
import FilledInput from '@mui/material/FilledInput';
import OutlinedInput from '@mui/material/OutlinedInput';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/components/iconify';

import { ComponentBox } from '../../layout';

// ----------------------------------------------------------------------

const CURRENCIES = [
  { value: 'USD', label: '$' },
  { value: 'EUR', label: '€' },
  { value: 'BTC', label: '฿' },
  { value: 'JPY', label: '¥' },
];

// ----------------------------------------------------------------------

interface State {
  amount: string;
  password: string;
  weight: string;
  weightRange: string;
  showPassword: boolean;
}

type Props = {
  variant: 'filled' | 'outlined' | 'standard';
};

export function TextFieldVariant({ variant }: Props) {
  const [currency, setCurrency] = useState('EUR');

  const [values, setValues] = useState<State>({
    amount: '',
    password: '',
    weight: '',
    weightRange: '',
    showPassword: false,
  });

  const handleChange = useCallback(
    (prop: keyof State) => (event: React.ChangeEvent<HTMLInputElement>) => {
      setValues((prev) => ({ ...prev, [prop]: event.target.value }));
    },
    []
  );

  const handleChangeCurrency = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    setCurrency(event.target.value);
  }, []);

  const handleShowPassword = useCallback(() => {
    setValues((prev) => ({ ...prev, showPassword: !prev.showPassword }));
  }, []);

  const textFieldProps: Pick<TextFieldProps, 'variant' | 'fullWidth'> = {
    variant,
    fullWidth: true,
  };

  return (
    <>
      <ComponentBox title="General">
        <TextField {...textFieldProps} label="Inactive" />
        <TextField {...textFieldProps} required label="Activated" defaultValue="2Minimal" />
        <TextField
          {...textFieldProps}
          type="password"
          label="Password"
          autoComplete="current-password"
          defaultValue="2Minimal"
          slotProps={{
            inputLabel: { shrink: true },
          }}
        />
        <TextField {...textFieldProps} disabled label="Disabled" defaultValue="2Minimal" />

        {(variant === 'filled' && (
          <FilledInput fullWidth hiddenLabel placeholder="FilledInput" />
        )) ||
          (variant === 'outlined' && <OutlinedInput fullWidth placeholder="OutlinedInput" />) || (
            <Input fullWidth placeholder="Input" />
          )}
      </ComponentBox>

      <ComponentBox title="With adornments">
        <TextField
          {...textFieldProps}
          label="Enabled"
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="solar:user-rounded-bold" width={24} />
                </InputAdornment>
              ),
            },
          }}
        />

        <TextField
          {...textFieldProps}
          disabled
          label="Disabled"
          defaultValue="Default value"
          helperText={
            <>
              <Iconify icon="eva:info-outline" />
              Helper text
            </>
          }
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="solar:user-rounded-bold" width={24} />
                </InputAdornment>
              ),
            },
          }}
        />

        <TextField
          {...textFieldProps}
          label="With normal TextField"
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start" disableTypography sx={{ typography: 'subtitle2' }}>
                  Kg
                </InputAdornment>
              ),
              endAdornment: (
                <InputAdornment position="end" sx={{ mr: -0.5 }}>
                  <Button variant="contained">Action</Button>
                </InputAdornment>
              ),
            },
          }}
        />

        <TextField
          {...textFieldProps}
          value={values.weight}
          onChange={handleChange('weight')}
          hiddenLabel={variant === 'filled'}
          placeholder="End adornment"
          helperText="Weight"
          slotProps={{
            input: {
              endAdornment: (
                <InputAdornment position="end" disableTypography sx={{ typography: 'subtitle2' }}>
                  Kg
                </InputAdornment>
              ),
            },
          }}
        />

        <TextField
          {...textFieldProps}
          type={values.showPassword ? 'text' : 'password'}
          value={values.password}
          onChange={handleChange('password')}
          label="Password"
          slotProps={{
            input: {
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="solar:user-rounded-bold" width={24} />
                </InputAdornment>
              ),
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton
                    edge="end"
                    onClick={handleShowPassword}
                    onMouseDown={(event: React.MouseEvent<HTMLButtonElement>) =>
                      event.preventDefault()
                    }
                    onMouseUp={(event: React.MouseEvent<HTMLButtonElement>) =>
                      event.preventDefault()
                    }
                  >
                    {values.showPassword ? (
                      <Iconify icon="solar:eye-bold" width={24} />
                    ) : (
                      <Iconify icon="solar:eye-closed-bold" width={24} />
                    )}
                  </IconButton>
                </InputAdornment>
              ),
            },
          }}
        />
      </ComponentBox>

      <ComponentBox title="With helper text">
        <TextField
          {...textFieldProps}
          label="Helper text"
          defaultValue="2Minimal"
          helperText={
            <>
              <Iconify icon="eva:info-outline" />
              Helper text
            </>
          }
        />

        <TextField
          {...textFieldProps}
          error
          label="Error"
          defaultValue="2Minimal"
          helperText="Error text"
        />
      </ComponentBox>

      <ComponentBox title="Type">
        <TextField
          {...textFieldProps}
          type="password"
          label="Password"
          autoComplete="current-password"
        />

        <TextField {...textFieldProps} label="Search" type="search" />
      </ComponentBox>

      <ComponentBox title="Sizes">
        <TextField {...textFieldProps} label="Size" size="small" defaultValue="Small" />
        <TextField {...textFieldProps} label="Size" defaultValue="Medium" />
      </ComponentBox>

      <ComponentBox title="Select">
        <TextField
          {...textFieldProps}
          select
          label="Select"
          value={currency}
          onChange={handleChangeCurrency}
          helperText="Please select your currency"
          slotProps={{
            htmlInput: { id: `${variant}-currency-select` },
            inputLabel: { htmlFor: `${variant}-currency-select` },
          }}
        >
          {CURRENCIES.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </TextField>

        <TextField
          {...textFieldProps}
          select
          size="small"
          value={currency}
          label="Native select"
          onChange={handleChangeCurrency}
          helperText="Please select your currency"
          slotProps={{
            select: { native: true },
            htmlInput: { id: `${variant}-currency-native-select` },
            inputLabel: { htmlFor: `${variant}-currency-native-select` },
          }}
        >
          {CURRENCIES.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </TextField>

        <FormControl {...textFieldProps} size="small">
          <InputLabel htmlFor={`form-control-${variant}-select`}>Form control select</InputLabel>
          <Select
            label="Form control select"
            value={currency}
            onChange={(event) => setCurrency(event.target.value)}
            inputProps={{ id: `form-control-${variant}-select` }}
          >
            {CURRENCIES.map((option) => (
              <MenuItem key={option.value} value={option.value}>
                {option.label}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl {...textFieldProps} size="small">
          <InputLabel htmlFor={`form-control-${variant}-native-select`}>
            Form control select (native)
          </InputLabel>
          <Select
            native
            label="Form control select (native)"
            value={currency}
            onChange={(event) => setCurrency(event.target.value)}
            inputProps={{ id: `form-control-${variant}-native-select` }}
          >
            {CURRENCIES.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </Select>
        </FormControl>
      </ComponentBox>

      <ComponentBox title="Multiline">
        <TextField {...textFieldProps} multiline maxRows={4} label="Multiline" value="Controlled" />

        <TextField
          {...textFieldProps}
          multiline
          placeholder="Placeholder"
          label="Multiline placeholder"
        />

        <TextField
          {...textFieldProps}
          rows={4}
          multiline
          label="Multiline"
          defaultValue="Default value"
        />

        <TextField
          {...textFieldProps}
          hiddenLabel={variant === 'filled'}
          rows={4}
          multiline
          defaultValue="No label"
        />
      </ComponentBox>
    </>
  );
}
