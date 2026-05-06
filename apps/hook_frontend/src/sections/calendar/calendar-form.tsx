import type { ICalendarEvent } from 'src/types/calendar';

import * as z from 'zod';
import { useCallback } from 'react';
import { uuidv4 } from 'minimal-shared/utils';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, Controller } from 'react-hook-form';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import DialogActions from '@mui/material/DialogActions';

import { fIsAfter } from 'src/utils/format-time';

import { createEvent, updateEvent, deleteEvent } from 'src/actions/calendar';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Scrollbar } from 'src/components/scrollbar';
import { Form, Field } from 'src/components/hook-form';
import { ColorPicker } from 'src/components/color-utils';

// ----------------------------------------------------------------------

export type EventSchemaType = z.infer<typeof EventSchema>;

export const EventSchema = z.object({
  title: z
    .string()
    .min(1, { error: 'Title is required!' })
    .max(100, { error: 'Title must be less than 100 characters' }),
  description: z
    .string()
    .min(1, { error: 'Description is required!' })
    .min(50, { error: 'Description must be at least 50 characters' }),
  // Not required
  color: z.string(),
  allDay: z.boolean(),
  start: z.union([z.string(), z.number()]),
  end: z.union([z.string(), z.number()]),
});

// ----------------------------------------------------------------------

type Props = {
  colorOptions: string[];
  onClose: () => void;
  currentEvent?: ICalendarEvent;
};

export function CalendarForm({ currentEvent, colorOptions, onClose }: Props) {
  const methods = useForm({
    mode: 'all',
    resolver: zodResolver(EventSchema),
    defaultValues: currentEvent,
  });

  const {
    reset,
    watch,
    control,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const values = watch();

  const dateError = fIsAfter(values.start, values.end);

  const onSubmit = handleSubmit(async (data) => {
    const eventData = {
      id: currentEvent?.id ? currentEvent?.id : uuidv4(),
      color: data?.color,
      title: data?.title,
      allDay: data?.allDay,
      description: data?.description,
      end: data?.end,
      start: data?.start,
    };

    try {
      if (!dateError) {
        if (currentEvent?.id) {
          await updateEvent(eventData);
          toast.success('Update success!');
        } else {
          await createEvent(eventData);
          toast.success('Create success!');
        }
        onClose();
        reset();
      }
    } catch (error) {
      console.error(error);
    }
  });

  const onDelete = useCallback(async () => {
    try {
      await deleteEvent(`${currentEvent?.id}`);
      toast.success('Delete success!');
      onClose();
    } catch (error) {
      console.error(error);
    }
  }, [currentEvent?.id, onClose]);

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Scrollbar sx={{ p: 3, bgcolor: 'background.neutral' }}>
        <Stack spacing={3}>
          <Field.Text name="title" label="Title" />
          <Field.Text name="description" label="Description" multiline rows={3} />
          <Field.Switch name="allDay" label="All day" />
          <Field.DateTimePicker name="start" label="Start date" />
          <Field.DateTimePicker
            name="end"
            label="End date"
            slotProps={{
              textField: {
                error: dateError,
                helperText: dateError ? 'End date must be later than start date' : null,
              },
            }}
          />

          <Controller
            name="color"
            control={control}
            render={({ field }) => (
              <ColorPicker
                value={field.value as string}
                onChange={(color) => field.onChange(color as string)}
                options={colorOptions}
              />
            )}
          />
        </Stack>
      </Scrollbar>

      <DialogActions sx={{ flexShrink: 0 }}>
        {!!currentEvent?.id && (
          <Tooltip title="Delete event">
            <IconButton color="error" onClick={onDelete} edge="start">
              <Iconify icon="solar:trash-bin-trash-bold" />
            </IconButton>
          </Tooltip>
        )}

        <Box component="span" sx={{ flexGrow: 1 }} />

        <Button variant="outlined" color="inherit" onClick={onClose}>
          Cancel
        </Button>
        <Button type="submit" variant="contained" loading={isSubmitting} disabled={dateError}>
          {currentEvent?.id ? 'Save changes' : 'Create'}
        </Button>
      </DialogActions>
    </Form>
  );
}
