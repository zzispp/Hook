'use client';

import type { SystemUser } from 'src/types/rbac';
import type { UserGroup } from 'src/types/user-group';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';
import ListItemText from '@mui/material/ListItemText';

import { useTranslate } from 'src/locales/use-locales';

import { Iconify } from 'src/components/iconify';

import { TextFieldRow, ManagementDialog } from './shared';
import { displayUserGroup, userGroupSelectionLabel } from './user-group-utils';
import { useUserGroupAssignmentDialogState } from './user-group-assignment-state';

type Props = {
  initialGroup: UserGroup | null;
  displayGroups: UserGroup[];
  groups: UserGroup[];
  onClose: () => void;
  onAssigned: () => void;
};

export function UserGroupAssignmentDialog({
  initialGroup,
  displayGroups,
  groups,
  onClose,
  onAssigned,
}: Props) {
  const state = useUserGroupAssignmentDialogState({
    initialGroup,
    displayGroups,
    groups,
    onClose,
    onAssigned,
  });

  return (
    <ManagementDialog
      open={!!initialGroup}
      title={state.t('dialogs.assignUserGroup')}
      submitting={state.submitting}
      submitDisabled={!state.canSubmit}
      onClose={state.close}
      onSubmit={state.submit}
    >
      <UserSearchField
        value={state.user}
        inputValue={state.search}
        loading={state.usersLoading}
        options={state.users}
        groups={state.visibleGroups}
        onInputChange={state.setSearch}
        onChange={state.setUser}
      />
      <CurrentGroupField user={state.user} groups={state.visibleGroups} />
      <TargetGroupField
        value={state.targetGroupCodes}
        groups={state.targetGroups}
        onChange={state.setTargetCodes}
      />
      <AssignmentPreview
        user={state.user}
        targetCodes={state.targetGroupCodes}
        groups={state.visibleGroups}
      />
    </ManagementDialog>
  );
}

function UserSearchField({
  value,
  inputValue,
  loading,
  options,
  groups,
  onInputChange,
  onChange,
}: {
  value: SystemUser | null;
  inputValue: string;
  loading: boolean;
  options: SystemUser[];
  groups: UserGroup[];
  onInputChange: (value: string) => void;
  onChange: (user: SystemUser | null) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <Autocomplete
      fullWidth
      loading={loading}
      value={value}
      inputValue={inputValue}
      options={options}
      filterOptions={(items) => items}
      getOptionLabel={userLabel}
      isOptionEqualToValue={(option, current) => option.id === current.id}
      noOptionsText={t('common.noResults')}
      onInputChange={(_event, nextValue) => onInputChange(nextValue)}
      onChange={(_event, nextValue) => onChange(nextValue)}
      renderInput={(params) => (
        <TextField {...params} label={t('fields.assignUser')} placeholder={t('filters.searchUsers')} />
      )}
      renderOption={(props, option) => (
        <MenuItem {...props} key={option.id}>
          <Stack spacing={0.25}>
            <Typography variant="subtitle2">{option.username}</Typography>
            <Typography variant="caption" color="text.secondary">
              {option.email} · {userGroupSelectionLabel(option.group_codes, groups, t)}
            </Typography>
          </Stack>
        </MenuItem>
      )}
    />
  );
}

function CurrentGroupField({ user, groups }: { user: SystemUser | null; groups: UserGroup[] }) {
  const { t } = useTranslate('admin');

  return (
    <TextFieldRow
      disabled
      label={t('fields.currentUserGroup')}
      value={user ? userGroupSelectionLabel(user.group_codes, groups, t) : ''}
      placeholder={t('userGroups.selectUserForAssignment')}
      onChange={() => undefined}
    />
  );
}

function TargetGroupField({
  value,
  groups,
  onChange,
}: {
  value: string[];
  groups: UserGroup[];
  onChange: (value: string[]) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TextField
      select
      required
      fullWidth
      label={t('fields.targetUserGroup')}
      value={value}
      SelectProps={{
        multiple: true,
        renderValue: (selected) => userGroupSelectionLabel(selected as string[], groups, t),
      }}
      onChange={(event) => onChange(selectedValues(event.target.value))}
    >
      {groups.length === 0 ? (
        <MenuItem disabled value="">
          {t('userGroups.noActiveGroups')}
        </MenuItem>
      ) : null}
      {groups.map((group) => (
        <MenuItem key={group.code} value={group.code}>
          <Checkbox checked={value.includes(group.code)} />
          <ListItemText primary={group.name} secondary={group.code} />
        </MenuItem>
      ))}
    </TextField>
  );
}

function AssignmentPreview({
  user,
  targetCodes,
  groups,
}: {
  user: SystemUser | null;
  targetCodes: string[];
  groups: UserGroup[];
}) {
  const { t } = useTranslate('admin');
  if (!user) {
    return <Alert severity="info">{t('userGroups.selectUserForAssignment')}</Alert>;
  }

  const currentName = userGroupSelectionLabel(user.group_codes, groups, t);
  const targetName =
    targetCodes.length > 0 ? userGroupSelectionLabel(targetCodes, groups, t) : t('common.none');

  return (
    <Box sx={{ p: 2, borderRadius: 1, bgcolor: 'background.neutral' }}>
      <Stack spacing={1.25}>
        <Typography variant="subtitle2">{t('userGroups.assignmentPreview')}</Typography>
        <Stack direction="row" spacing={1} alignItems="center" sx={{ flexWrap: 'wrap' }}>
          <Chip label={currentName} variant="soft" color="default" />
          <Iconify icon="solar:double-alt-arrow-right-bold-duotone" sx={{ color: 'text.secondary' }} />
          {targetCodes.length === 0 ? (
            <Chip label={targetName} variant="soft" color="info" />
          ) : (
            targetCodes.map((code) => (
              <Chip key={code} label={displayUserGroup(code, groups)} variant="soft" color="info" />
            ))
          )}
        </Stack>
        <Typography variant="body2" color="text.secondary">
          {t('userGroups.assignmentPreviewText', {
            username: user.username,
            from: currentName,
            to: targetName,
          })}
        </Typography>
      </Stack>
    </Box>
  );
}

function userLabel(user: SystemUser) {
  return `${user.username} (${user.email})`;
}

function selectedValues(value: string | string[]) {
  return Array.isArray(value) ? value : value.split(',').filter(Boolean);
}
