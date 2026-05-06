import type { IJobItem } from 'src/types/job';

import * as z from 'zod';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, Controller } from 'react-hook-form';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import Switch from '@mui/material/Switch';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import ButtonBase from '@mui/material/ButtonBase';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import FormControlLabel from '@mui/material/FormControlLabel';

import { paths } from 'src/routes/paths';
import { useRouter } from 'src/routes/hooks';

import {
  _roles,
  JOB_SKILL_OPTIONS,
  JOB_BENEFIT_OPTIONS,
  JOB_EXPERIENCE_OPTIONS,
  JOB_EMPLOYMENT_TYPE_OPTIONS,
  JOB_WORKING_SCHEDULE_OPTIONS,
} from 'src/_mock';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Form, Field, schemaUtils } from 'src/components/hook-form';

// ----------------------------------------------------------------------

export type JobCreateSchemaType = z.infer<typeof JobCreateSchema>;

export const JobCreateSchema = z.object({
  title: z.string().min(1, { error: 'Title is required!' }),
  content: schemaUtils.editor().min(100, { error: 'Content must be at least 100 characters' }),
  employmentTypes: z.string().array().min(1, { error: 'Choose at least one option!' }),
  role: schemaUtils.nullableInput(z.string().min(1, { error: 'Role is required!' }), {
    error: 'Role is required!',
  }),
  skills: z.string().array().min(1, { error: 'Choose at least one option!' }),
  workingSchedule: z.string().array().min(1, { error: 'Choose at least one option!' }),
  locations: z.string().array().min(1, { error: 'Choose at least one option!' }),
  expiredDate: schemaUtils.date({ error: { required: 'Expired date is required!' } }),
  salary: z.object({
    price: schemaUtils.nullableInput(z.coerce.number().min(1, { error: 'Price is required!' }), {
      error: 'Price is required!',
    }),
    // Not required
    type: z.string(),
    negotiable: z.boolean(),
  }),
  benefits: z.string().array().min(1, { error: 'Choose at least one option!' }),
  // Not required
  experience: z.string(),
});

// ----------------------------------------------------------------------

type Props = {
  currentJob?: IJobItem;
};

export function JobCreateEditForm({ currentJob }: Props) {
  const router = useRouter();

  const openDetails = useBoolean(true);
  const openProperties = useBoolean(true);

  const defaultValues: JobCreateSchemaType = {
    title: '',
    content: '',
    employmentTypes: [],
    experience: '1 year exp',
    role: _roles[1],
    skills: [],
    workingSchedule: [],
    locations: [],
    expiredDate: null,
    salary: { type: 'Hourly', price: null, negotiable: false },
    benefits: [],
  };

  const methods = useForm({
    mode: 'all',
    resolver: zodResolver(JobCreateSchema),
    defaultValues,
    values: currentJob,
  });

  const {
    reset,
    control,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      toast.success(currentJob ? 'Update success!' : 'Create success!');
      router.push(paths.dashboard.job.root);
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  const renderCollapseButton = (value: boolean, onToggle: () => void) => (
    <IconButton onClick={onToggle}>
      <Iconify icon={value ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'} />
    </IconButton>
  );

  const renderDetails = () => (
    <Card>
      <CardHeader
        title="Details"
        subheader="Title, short description, image..."
        action={renderCollapseButton(openDetails.value, openDetails.onToggle)}
        sx={{ mb: 3 }}
      />

      <Collapse in={openDetails.value}>
        <Divider />

        <Stack spacing={3} sx={{ p: 3 }}>
          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Title</Typography>
            <Field.Text name="title" placeholder="Ex: Software engineer..." />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Content</Typography>
            <Field.Editor name="content" sx={{ maxHeight: 480 }} />
          </Stack>
        </Stack>
      </Collapse>
    </Card>
  );

  const renderProperties = () => (
    <Card>
      <CardHeader
        title="Properties"
        subheader="Additional functions and attributes..."
        action={renderCollapseButton(openProperties.value, openProperties.onToggle)}
        sx={{ mb: 3 }}
      />

      <Collapse in={openProperties.value}>
        <Divider />

        <Stack spacing={3} sx={{ p: 3 }}>
          <Stack spacing={1}>
            <Typography variant="subtitle2">Employment type</Typography>
            <Field.MultiCheckbox
              row
              name="employmentTypes"
              options={JOB_EMPLOYMENT_TYPE_OPTIONS}
              sx={{ gap: 4 }}
            />
          </Stack>

          <Stack spacing={1}>
            <Typography variant="subtitle2">Experience</Typography>
            <Field.RadioGroup
              row
              name="experience"
              options={JOB_EXPERIENCE_OPTIONS}
              sx={{ gap: 4 }}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Role</Typography>
            <Field.Autocomplete
              name="role"
              autoHighlight
              options={_roles.map((option) => option)}
              getOptionLabel={(option) => option}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Skills</Typography>
            <Field.Autocomplete
              name="skills"
              placeholder="+ Skills"
              multiple
              disableCloseOnSelect
              options={JOB_SKILL_OPTIONS.map((option) => option)}
              getOptionLabel={(option) => option}
              slotProps={{
                chip: { color: 'info' },
              }}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Working schedule</Typography>
            <Field.Autocomplete
              name="workingSchedule"
              placeholder="+ Schedule"
              multiple
              disableCloseOnSelect
              options={JOB_WORKING_SCHEDULE_OPTIONS.map((option) => option)}
              getOptionLabel={(option) => option}
              slotProps={{
                chip: { color: 'info' },
              }}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Locations</Typography>
            <Field.CountrySelect
              multiple
              name="locations"
              placeholder="+ Locations"
              slotProps={{
                chip: { color: 'info' },
              }}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Expired</Typography>

            <Field.DatePicker name="expiredDate" />
          </Stack>

          <Stack spacing={2}>
            <Typography variant="subtitle2">Salary</Typography>

            <Controller
              name="salary.type"
              control={control}
              render={({ field }) => (
                <Box sx={{ gap: 2, display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)' }}>
                  {[
                    {
                      label: 'Hourly',
                      icon: <Iconify icon="solar:clock-circle-bold" width={32} sx={{ mb: 2 }} />,
                    },
                    {
                      label: 'Custom',
                      icon: <Iconify icon="solar:wad-of-money-bold" width={32} sx={{ mb: 2 }} />,
                    },
                  ].map((item) => (
                    <Paper
                      component={ButtonBase}
                      variant="outlined"
                      key={item.label}
                      onClick={() => field.onChange(item.label)}
                      sx={{
                        p: 2.5,
                        borderRadius: 1,
                        typography: 'subtitle2',
                        flexDirection: 'column',
                        ...(item.label === field.value && {
                          borderWidth: 2,
                          borderColor: 'text.primary',
                        }),
                      }}
                    >
                      {item.icon}
                      {item.label}
                    </Paper>
                  ))}
                </Box>
              )}
            />

            <Field.Text
              name="salary.price"
              placeholder="0.00"
              type="number"
              slotProps={{
                input: {
                  startAdornment: (
                    <InputAdornment position="start" sx={{ mr: 0.75 }}>
                      <Box component="span" sx={{ color: 'text.disabled' }}>
                        $
                      </Box>
                    </InputAdornment>
                  ),
                },
              }}
            />

            <Field.Switch name="salary.negotiable" label="Salary is negotiable" />
          </Stack>

          <Stack spacing={1}>
            <Typography variant="subtitle2">Benefits</Typography>
            <Field.MultiCheckbox
              name="benefits"
              options={JOB_BENEFIT_OPTIONS}
              sx={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)' }}
            />
          </Stack>
        </Stack>
      </Collapse>
    </Card>
  );

  const renderActions = () => (
    <Box sx={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center' }}>
      <FormControlLabel
        label="Publish"
        control={<Switch defaultChecked slotProps={{ input: { id: 'publish-switch' } }} />}
        sx={{ flexGrow: 1, pl: 3 }}
      />

      <Button type="submit" variant="contained" size="large" loading={isSubmitting} sx={{ ml: 2 }}>
        {!currentJob ? 'Create job' : 'Save changes'}
      </Button>
    </Box>
  );

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Stack spacing={{ xs: 3, md: 5 }} sx={{ mx: 'auto', maxWidth: { xs: 720, xl: 880 } }}>
        {renderDetails()}
        {renderProperties()}
        {renderActions()}
      </Stack>
    </Form>
  );
}
