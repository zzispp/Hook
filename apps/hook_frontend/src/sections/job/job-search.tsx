import type { Theme, SxProps } from '@mui/material/styles';
import type { IJobItem } from 'src/types/job';

import parse from 'autosuggest-highlight/parse';
import match from 'autosuggest-highlight/match';
import { useDebounce } from 'minimal-shared/hooks';
import { useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Link, { linkClasses } from '@mui/material/Link';
import InputAdornment from '@mui/material/InputAdornment';
import CircularProgress from '@mui/material/CircularProgress';
import Autocomplete, { autocompleteClasses } from '@mui/material/Autocomplete';

import { useRouter } from 'src/routes/hooks';
import { RouterLink } from 'src/routes/components';

import { _jobs } from 'src/_mock';

import { Iconify } from 'src/components/iconify';
import { SearchNotFound } from 'src/components/search-not-found';

// ----------------------------------------------------------------------

type Props = {
  sx?: SxProps<Theme>;
  redirectPath: (id: string) => string;
};

export function JobSearch({ redirectPath, sx }: Props) {
  const router = useRouter();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedItem, setSelectedItem] = useState<IJobItem | null>(null);

  const debouncedQuery = useDebounce(searchQuery);
  const { searchResults: options, searchLoading: loading } = useSearchData(debouncedQuery);

  const handleChange = useCallback(
    (item: IJobItem | null) => {
      setSelectedItem(item);
      if (item) {
        router.push(redirectPath(item.id));
      }
    },
    [redirectPath, router]
  );

  const paperStyles: SxProps<Theme> = {
    width: 320,
    [`& .${autocompleteClasses.listbox}`]: {
      [`& .${autocompleteClasses.option}`]: {
        p: 0,
        [`& .${linkClasses.root}`]: {
          px: 1,
          py: 0.75,
          width: 1,
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
              href={redirectPath(option.id)}
              color="inherit"
              underline="none"
            >
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

// ----------------------------------------------------------------------

function useSearchData(searchQuery: string) {
  const [searchResults, setSearchResults] = useState<IJobItem[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);

  const fetchSearchResults = useCallback(async () => {
    setSearchLoading(true);

    try {
      await new Promise((resolve) => setTimeout(resolve, 500));

      const results = _jobs.filter(({ title }) =>
        [title].some((field) => field?.toLowerCase().includes(searchQuery.toLowerCase()))
      );

      setSearchResults(results);
    } catch (error) {
      console.error(error);
    } finally {
      setSearchLoading(false);
    }
  }, [searchQuery]);

  useEffect(() => {
    if (searchQuery) {
      fetchSearchResults();
    } else {
      setSearchResults([]);
    }
  }, [fetchSearchResults, searchQuery]);

  return { searchResults, searchLoading };
}
