import { useFormContext } from 'react-hook-form';

import Box from '@mui/material/Box';
import MenuItem from '@mui/material/MenuItem';

import { Field } from 'src/components/hook-form';

// ----------------------------------------------------------------------

const STATUS_OPTIONS = ['paid', 'pending', 'overdue', 'draft'] as const;

// ----------------------------------------------------------------------

export function InvoiceCreateEditStatusDate() {
  const { watch } = useFormContext();

  const values = watch();

  return (
    <Box
      sx={{
        p: 3,
        gap: 2,
        display: 'flex',
        bgcolor: 'background.neutral',
        flexDirection: { xs: 'column', sm: 'row' },
      }}
    >
      <Field.Text
        disabled
        name="invoiceNumber"
        label="Invoice number"
        value={values.invoiceNumber}
      />

      <Field.Select
        fullWidth
        name="status"
        label="Status"
        slotProps={{ inputLabel: { shrink: true } }}
      >
        {STATUS_OPTIONS.map((option) => (
          <MenuItem key={option} value={option} sx={{ textTransform: 'capitalize' }}>
            {option}
          </MenuItem>
        ))}
      </Field.Select>

      <Field.DatePicker name="createDate" label="Date create" />
      <Field.DatePicker name="dueDate" label="Due date" />
    </Box>
  );
}
