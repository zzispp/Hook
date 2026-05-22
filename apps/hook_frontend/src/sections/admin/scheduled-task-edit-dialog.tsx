'use client';

import type { ScheduledTask } from 'src/types/scheduler';
import type { Translate, TaskFormState } from './scheduled-tasks-utils';

import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { TextFieldRow } from './shared';

export function ScheduledTaskEditor({
  form,
  task,
  t,
  onChange,
}: {
  form: TaskFormState;
  task: ScheduledTask;
  t: Translate;
  onChange: React.Dispatch<React.SetStateAction<TaskFormState | null>>;
}) {
  return (
    <Stack spacing={2.5}>
      <FormControlLabel
        control={
          <Switch
            checked={form.enabled}
            onChange={(event) =>
              onChange((current) =>
                current
                  ? {
                      ...current,
                      enabled: event.target.checked,
                    }
                  : current
              )
            }
          />
        }
        label={t('common.enabled')}
      />
      <TextFieldRow
        type="number"
        label={t('scheduledTasks.fields.intervalSeconds')}
        value={form.interval_seconds}
        onChange={(value) =>
          onChange((current) =>
            current
              ? {
                  ...current,
                  interval_seconds: value,
                }
              : current
          )
        }
      />
      <Divider />
      <Stack spacing={2}>
        {task.config_schema.length ? (
          task.config_schema.map((field) => (
            <TextFieldRow
              key={field.key}
              type="number"
              label={t(field.label_key)}
              value={form.config[field.key] ?? ''}
              helperText={field.unit_key ? t(field.unit_key) : undefined}
              onChange={(value) =>
                onChange((current) =>
                  current
                    ? {
                        ...current,
                        config: {
                          ...current.config,
                          [field.key]: value,
                        },
                      }
                    : current
                )
              }
            />
          ))
        ) : (
          <Paper variant="outlined" sx={{ p: 2 }}>
            <Typography variant="body2" color="text.secondary">
              {t('scheduledTasks.emptyConfig')}
            </Typography>
          </Paper>
        )}
      </Stack>
    </Stack>
  );
}
