'use client';

import type { TFunction } from 'i18next';
import type { HTMLAttributes } from 'react';
import type { SystemUser } from 'src/types/rbac';

import { useMemo, useState } from 'react';

import Stack from '@mui/material/Stack';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { useUsers } from 'src/actions/rbac';

type ReferrerSearchType = 'affiliate_code' | 'user';
type AffiliateCodeOption = {
  code: string;
  username: string;
  email: string;
};

const SEARCH_PAGE = 0;
const DISABLED_SEARCH_PAGE = -1;
const SEARCH_PAGE_SIZE = 20;

export function AffiliateReferrerSelector({
  t,
  value,
  active,
  excludedUserId,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  active: boolean;
  excludedUserId: string;
  onChange: (value: string) => void;
}) {
  const [searchType, setSearchType] = useState<ReferrerSearchType>('affiliate_code');
  const [searchText, setSearchText] = useState(value);
  const searchPage = active ? SEARCH_PAGE : DISABLED_SEARCH_PAGE;
  const users = useUsers(searchPage, SEARCH_PAGE_SIZE, { search: searchText.trim() || undefined });
  const candidates = useMemo(
    () => users.items.filter((user) => validReferrerCandidate(user, excludedUserId)),
    [excludedUserId, users.items]
  );
  const selectedUser = useMemo(
    () => candidates.find((user) => user.affiliate_code === value) ?? null,
    [candidates, value]
  );
  const loading = users.isLoading || users.isValidating;

  return (
    <ReferrerSelector
      t={t}
      type={searchType}
      value={value}
      selectedUser={selectedUser}
      inputValue={searchText}
      loading={loading}
      users={candidates}
      onTypeChange={(nextType) => {
        setSearchType(nextType);
        setSearchText(value);
      }}
      onInputChange={setSearchText}
      onAffCodeChange={(referrerAffCode) => {
        setSearchText(referrerAffCode);
        onChange(referrerAffCode);
      }}
      onUserChange={(user) => {
        onChange(user?.affiliate_code ?? '');
        setSearchText(user ? userLabel(user) : '');
      }}
    />
  );
}

function ReferrerSelector({
  t,
  type,
  value,
  selectedUser,
  inputValue,
  loading,
  users,
  onTypeChange,
  onInputChange,
  onAffCodeChange,
  onUserChange,
}: {
  t: TFunction<'admin'>;
  type: ReferrerSearchType;
  value: string;
  selectedUser: SystemUser | null;
  inputValue: string;
  loading: boolean;
  users: SystemUser[];
  onTypeChange: (value: ReferrerSearchType) => void;
  onInputChange: (value: string) => void;
  onAffCodeChange: (value: string) => void;
  onUserChange: (user: SystemUser | null) => void;
}) {
  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
      <TextField
        select
        label={t('adminAffiliates.fields.referrerSearchType')}
        value={type}
        sx={{ minWidth: { sm: 160 } }}
        onChange={(event) => onTypeChange(event.target.value as ReferrerSearchType)}
      >
        <MenuItem value="affiliate_code">{t('adminAffiliates.fields.referrerSearchTypeAffCode')}</MenuItem>
        <MenuItem value="user">{t('adminAffiliates.fields.referrerSearchTypeUser')}</MenuItem>
      </TextField>
      {type === 'affiliate_code' ? (
        <AffiliateCodeSearch
          t={t}
          value={value}
          inputValue={inputValue}
          loading={loading}
          users={users}
          onInputChange={onInputChange}
          onChange={onAffCodeChange}
        />
      ) : (
        <UserSearch
          t={t}
          value={selectedUser}
          inputValue={inputValue}
          loading={loading}
          users={users}
          onInputChange={onInputChange}
          onChange={onUserChange}
        />
      )}
    </Stack>
  );
}

function AffiliateCodeSearch({
  t,
  value,
  inputValue,
  loading,
  users,
  onInputChange,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: string;
  inputValue: string;
  loading: boolean;
  users: SystemUser[];
  onInputChange: (value: string) => void;
  onChange: (value: string) => void;
}) {
  return (
    <Autocomplete
      freeSolo
      fullWidth
      loading={loading}
      value={value}
      inputValue={inputValue}
      options={users.map(affiliateCodeOption)}
      filterOptions={(items) => items}
      getOptionLabel={affiliateCodeLabel}
      isOptionEqualToValue={(option, current) => normalizeAffCodeValue(option) === normalizeAffCodeValue(current)}
      noOptionsText={t('common.noResults')}
      onInputChange={(_event, nextValue) => {
        onInputChange(nextValue);
        onChange(nextValue);
      }}
      onChange={(_event, nextValue) => onChange(normalizeAffCodeValue(nextValue))}
      renderInput={(params) => (
        <TextField {...params} required label={t('adminAffiliates.fields.newReferrerCode')} />
      )}
      renderOption={(props, option) => renderAffiliateCodeOption(props, option)}
    />
  );
}

function UserSearch({
  t,
  value,
  inputValue,
  loading,
  users,
  onInputChange,
  onChange,
}: {
  t: TFunction<'admin'>;
  value: SystemUser | null;
  inputValue: string;
  loading: boolean;
  users: SystemUser[];
  onInputChange: (value: string) => void;
  onChange: (user: SystemUser | null) => void;
}) {
  return (
    <Autocomplete
      fullWidth
      loading={loading}
      value={value}
      inputValue={inputValue}
      options={users}
      filterOptions={(items) => items}
      getOptionLabel={userLabel}
      isOptionEqualToValue={(option, current) => option.id === current.id}
      noOptionsText={t('common.noResults')}
      onInputChange={(_event, nextValue) => onInputChange(nextValue)}
      onChange={(_event, nextValue) => onChange(nextValue)}
      renderInput={(params) => (
        <TextField {...params} required label={t('adminAffiliates.fields.newReferrerUser')} />
      )}
      renderOption={(props, option) => (
        <MenuItem {...props} key={option.id}>
          <Stack spacing={0.25}>
            <Typography variant="subtitle2">{option.username}</Typography>
            <Typography variant="caption" color="text.secondary">
              {option.email} · {option.affiliate_code}
            </Typography>
          </Stack>
        </MenuItem>
      )}
    />
  );
}

function affiliateCodeOption(user: SystemUser): AffiliateCodeOption {
  return {
    code: user.affiliate_code,
    username: user.username,
    email: user.email,
  };
}

function renderAffiliateCodeOption(props: HTMLAttributes<HTMLLIElement>, option: string | AffiliateCodeOption) {
  const code = normalizeAffCodeValue(option);
  return (
    <MenuItem {...props} key={code}>
      {typeof option === 'string' ? (
        <Typography variant="subtitle2">{code}</Typography>
      ) : (
        <Stack spacing={0.25}>
          <Typography variant="subtitle2">{option.code}</Typography>
          <Typography variant="caption" color="text.secondary">
            {option.username} · {option.email}
          </Typography>
        </Stack>
      )}
    </MenuItem>
  );
}

function affiliateCodeLabel(option: string | AffiliateCodeOption) {
  return typeof option === 'string' ? option : option.code;
}

function normalizeAffCodeValue(value: string | AffiliateCodeOption | null) {
  if (!value) return '';
  return typeof value === 'string' ? value : value.code;
}

function validReferrerCandidate(user: SystemUser, excludedUserId: string) {
  return user.id !== excludedUserId && user.role === 'user';
}

function userLabel(user: SystemUser) {
  return `${user.username} (${user.email})`;
}
