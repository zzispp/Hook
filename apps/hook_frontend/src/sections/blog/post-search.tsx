import type { Theme, SxProps } from '@mui/material/styles';
import type { IPostItem } from 'src/types/blog';

import { useState, useCallback } from 'react';
import parse from 'autosuggest-highlight/parse';
import match from 'autosuggest-highlight/match';
import { useDebounce } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Avatar from '@mui/material/Avatar';
import TextField from '@mui/material/TextField';
import Link, { linkClasses } from '@mui/material/Link';
import InputAdornment from '@mui/material/InputAdornment';
import CircularProgress from '@mui/material/CircularProgress';
import Autocomplete, { autocompleteClasses, createFilterOptions } from '@mui/material/Autocomplete';

import { useRouter } from 'src/routes/hooks';
import { RouterLink } from 'src/routes/components';

import { useSearchPosts } from 'src/actions/blog';

import { Iconify } from 'src/components/iconify';
import { SearchNotFound } from 'src/components/search-not-found';

// ----------------------------------------------------------------------

type Props = {
  sx?: SxProps<Theme>;
  redirectPath: (title: string) => string;
};

export function PostSearch({ redirectPath, sx }: Props) {
  const router = useRouter();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedItem, setSelectedItem] = useState<IPostItem | null>(null);

  const debouncedQuery = useDebounce(searchQuery);
  const { searchResults: options, searchLoading: loading } = useSearchPosts(debouncedQuery);

  const handleChange = useCallback(
    (item: IPostItem | null) => {
      setSelectedItem(item);
      if (item) {
        router.push(redirectPath(item.title));
      }
    },
    [redirectPath, router]
  );

  const filterOptions = createFilterOptions({
    matchFrom: 'any',
    stringify: (option: IPostItem) => `${option.title} ${option.description}`,
  });

  const paperStyles: SxProps<Theme> = {
    width: 320,
    [`& .${autocompleteClasses.listbox}`]: {
      [`& .${autocompleteClasses.option}`]: {
        p: 0,
        [`& .${linkClasses.root}`]: {
          p: 0.75,
          gap: 1.5,
          width: 1,
          display: 'flex',
          alignItems: 'center',
        },
      },
    },
  };

  return (
    <Autocomplete
      autoHighlight
      popupIcon={null}
      loading={loading}
      options={options}
      value={selectedItem}
      filterOptions={filterOptions}
      onChange={(event, newValue) => handleChange(newValue)}
      onInputChange={(event, newValue) => setSearchQuery(newValue)}
      getOptionLabel={(option) => option.title}
      noOptionsText={<SearchNotFound query={debouncedQuery} />}
      isOptionEqualToValue={(option, value) => option.id === value.id}
      slotProps={{ paper: { sx: paperStyles } }}
      sx={[{ width: { xs: 1, sm: 260 } }, ...(Array.isArray(sx) ? sx : [sx])]}
      renderInput={(params) => (
        <TextField
          {...params}
          placeholder="Search..."
          slotProps={{
            input: {
              ...params.InputProps,
              startAdornment: (
                <InputAdornment position="start">
                  <Iconify icon="eva:search-fill" sx={{ ml: 1, color: 'text.disabled' }} />
                </InputAdornment>
              ),
              endAdornment: (
                <>
                  {loading ? <CircularProgress size={18} color="inherit" sx={{ mr: -3 }} /> : null}
                  {params.InputProps.endAdornment}
                </>
              ),
            },
          }}
        />
      )}
      renderOption={(props, option, state) => {
        const { key, ...otherProps } = props;
        const matches = match(option.title, state.inputValue, { insideWords: true });
        const parts = parse(option.title, matches);

        return (
          <li key={key} {...otherProps}>
            <Link
              component={RouterLink}
              href={redirectPath(option.title)}
              color="inherit"
              underline="none"
            >
              <Avatar
                alt={option.title}
                src={option.coverUrl}
                variant="rounded"
                sx={{
                  width: 48,
                  height: 48,
                  flexShrink: 0,
                  borderRadius: 1,
                }}
              />
              <div>
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
              </div>
            </Link>
          </li>
        );
      }}
    />
  );
}
