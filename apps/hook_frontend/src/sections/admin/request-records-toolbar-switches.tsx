import Stack from '@mui/material/Stack';
import Switch from '@mui/material/Switch';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

export function RequestRecordsToolbarSwitches({
  autoRefresh,
  timingExpanded,
  autoRefreshLabel,
  timingExpandedLabel,
  onAutoRefreshChange,
  onTimingExpandedChange,
}: {
  autoRefresh: boolean;
  timingExpanded: boolean;
  autoRefreshLabel: string;
  timingExpandedLabel: string;
  onAutoRefreshChange: (value: boolean) => void;
  onTimingExpandedChange: (value: boolean) => void;
}) {
  return (
    <Stack direction="row" spacing={1} sx={{ flexWrap: 'wrap' }}>
      <ToolbarSwitch
        value={autoRefresh}
        label={autoRefreshLabel}
        onChange={onAutoRefreshChange}
      />
      <ToolbarSwitch
        value={timingExpanded}
        label={timingExpandedLabel}
        onChange={onTimingExpandedChange}
      />
    </Stack>
  );
}

function ToolbarSwitch({
  value,
  label,
  onChange,
}: {
  value: boolean;
  label: string;
  onChange: (value: boolean) => void;
}) {
  return (
    <Stack direction="row" alignItems="center">
      <FormControlLabel
        control={<Switch checked={value} onChange={(event) => onChange(event.target.checked)} />}
        label={<Typography variant="body2">{label}</Typography>}
        sx={{ whiteSpace: 'nowrap' }}
      />
    </Stack>
  );
}
