import type { Theme, SxProps } from '@mui/material/styles';
import type { NavItemData } from './nav-config-components';

import { useState, useCallback } from 'react';
import parse from 'autosuggest-highlight/parse';
import match from 'autosuggest-highlight/match';
import { isEqualPath } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Link, { linkClasses } from '@mui/material/Link';
import InputAdornment from '@mui/material/InputAdornment';
import { inputBaseClasses } from '@mui/material/InputBase';
import Autocomplete, { autocompleteClasses } from '@mui/material/Autocomplete';

import { RouterLink } from 'src/routes/components';
import { useRouter, usePathname } from 'src/routes/hooks';

import { Iconify } from 'src/components/iconify';
import { SearchNotFound } from 'src/components/search-not-found';

// ----------------------------------------------------------------------

type NavSearchProps = {
  sx?: SxProps<Theme>;
  navData?: {
    title: string;
    items: NavItemData[];
  }[];
};

const paperStyles: SxProps<Theme> = {
  width: 240,
  [`& .${autocompleteClasses.listbox}`]: {
    [`& .${autocompleteClasses.option}`]: {
      p: 0,
      [`& .${linkClasses.root}`]: {
        px: 1,
        py: 0.5,
        width: 1,
      },
    },
  },
};

const textFieldStyles: SxProps<Theme> = {
  [`& .${inputBaseClasses.input}`]: {
    typography: 'body2',
    fontWeight: 'fontWeightMedium',
  },
};

export function NavSearch({ navData = [], sx }: NavSearchProps) {
  const router = useRouter();
  const pathname = usePathname();

  const options = navData?.map((section) => section.items).flat();
  const activeOption = options?.find((opt) => isEqualPath(opt.href, pathname));

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedItem, setSelectedItem] = useState<NavItemData | null>(activeOption || null);

  const handleChange = useCallback(
    (item: NavItemData | null) => {
      setSelectedItem(item);
      if (item) {
        router.push(item?.href);
      }
    },
    [router]
  );

  return (
    <Autocomplete
      sx={sx}
      autoHighlight
      disableClearable
      popupIcon={null}
      options={options}
      value={selectedItem as NavItemData}
      onChange={(event, newValue) => handleChange(newValue)}
      onInputChange={(event, newValue) => setSearchQuery(newValue)}
      getOptionLabel={(option) => option.name}
      noOptionsText={<SearchNotFound query={searchQuery} />}
      isOptionEqualToValue={(option, value) => option.name === value.name}
      slotProps={{ paper: { sx: paperStyles } }}
      renderInput={(params) => (
        <TextField
          {...params}
          hiddenLabel
          size="small"
          variant="filled"
          placeholder="Search..."
          slotProps={{
            input: {
              ...params.InputProps,
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="eva:search-fill" sx={{ color: 'text.disabled' }} />
                </InputAdornment>
              ),
            },
          }}
          sx={textFieldStyles}
        />
      )}
      renderOption={(props, option, state) => {
        const { key, ...otherProps } = props;
        const matches = match(option.name, state.inputValue, { insideWords: true });
        const parts = parse(option.name, matches);

        return (
          <li key={key} {...otherProps}>
            <Link component={RouterLink} href={option.href} color="inherit" underline="none">
              {parts.map((part, index) => (
                <Box
                  key={index}
                  component="span"
                  sx={{
                    typography: 'body2',
                    fontWeight: 'fontWeightMedium',
                    ...(part.highlight && {
                      color: 'primary.main',
                      fontWeight: 'fontWeightSemiBold',
                    }),
                  }}
                >
                  {part.text}
                </Box>
              ))}
            </Link>
          </li>
        );
      }}
    />
  );
}
