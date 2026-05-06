import type { ITourItem } from 'src/types/tour';

import * as z from 'zod';
import { useCallback } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Switch from '@mui/material/Switch';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import IconButton from '@mui/material/IconButton';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { paths } from 'src/routes/paths';
import { useRouter } from 'src/routes/hooks';

import { fIsAfter } from 'src/utils/format-time';

import { _tags, _tourGuides, TOUR_SERVICE_OPTIONS } from 'src/_mock';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Form, Field, schemaUtils } from 'src/components/hook-form';

// ----------------------------------------------------------------------

export type TourCreateSchemaType = z.infer<typeof TourCreateSchema>;

export const TourCreateSchema = z
  .object({
    name: z.string().min(1, { error: 'Name is required!' }),
    content: schemaUtils.editor().min(100, { error: 'Content must be at least 100 characters' }),
    images: schemaUtils.files({ error: 'Images is required!' }).min(2, {
      error: 'Must have at least 2 items!',
    }),
    tourGuides: z
      .array(
        z.object({
          id: z.string(),
          name: z.string(),
          avatarUrl: z.string(),
          phoneNumber: z.string(),
        })
      )
      .min(1, { error: 'Must have at least 1 guide!' }),
    available: z.object({
      startDate: schemaUtils.date({ error: { required: 'Start date is required!' } }),
      endDate: schemaUtils.date({ error: { required: 'End date is required!' } }),
    }),
    durations: z.string().min(1, { error: 'Durations is required!' }),
    destination: schemaUtils.nullableInput(
      z.string().min(1, { error: 'Destination is required!' }),
      {
        error: 'Destination is required!',
      }
    ),
    services: z.string().array().min(2, { error: 'Must have at least 2 items!' }),
    tags: z.string().array().min(2, { error: 'Must have at least 2 items!' }),
  })
  .refine((val) => !fIsAfter(val.available.startDate, val.available.endDate), {
    error: 'End date cannot be earlier than start date!',
    path: ['available.endDate'],
  });

// ----------------------------------------------------------------------

type Props = {
  currentTour?: ITourItem;
};

export function TourCreateEditForm({ currentTour }: Props) {
  const router = useRouter();

  const openDetails = useBoolean(true);
  const openProperties = useBoolean(true);

  const defaultValues: TourCreateSchemaType = {
    name: '',
    content: '',
    images: [],
    tourGuides: [],
    available: {
      startDate: null,
      endDate: null,
    },
    durations: '',
    destination: '',
    services: [],
    tags: [],
  };

  const methods = useForm({
    mode: 'all',
    resolver: zodResolver(TourCreateSchema),
    defaultValues,
    values: currentTour,
  });

  const {
    watch,
    reset,
    setValue,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const values = watch();

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      toast.success(currentTour ? 'Update success!' : 'Create success!');
      router.push(paths.dashboard.tour.root);
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  const handleRemoveFile = useCallback(
    (inputFile: File | string) => {
      const filtered = values.images && values.images?.filter((file) => file !== inputFile);
      setValue('images', filtered, { shouldValidate: true });
    },
    [setValue, values.images]
  );

  const handleRemoveAllFiles = useCallback(() => {
    setValue('images', [], { shouldValidate: true });
  }, [setValue]);

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
            <Typography variant="subtitle2">Name</Typography>
            <Field.Text name="name" placeholder="Ex: Adventure seekers expedition..." />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Content</Typography>
            <Field.Editor name="content" sx={{ maxHeight: 480 }} />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Images</Typography>
            <Field.Upload
              multiple
              name="images"
              maxSize={3145728}
              onRemove={handleRemoveFile}
              onRemoveAll={handleRemoveAllFiles}
              onUpload={() => console.info('ON UPLOAD')}
            />
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
          <div>
            <Typography variant="subtitle2" sx={{ mb: 1.5 }}>
              Tour guide
            </Typography>

            <Field.Autocomplete
              multiple
              name="tourGuides"
              placeholder="+ Tour guides"
              disableCloseOnSelect
              options={_tourGuides}
              getOptionLabel={(option) => option.name}
              isOptionEqualToValue={(option, value) => option.id === value.id}
              renderOption={(props, option) => {
                const { key, ...otherProps } = props;

                return (
                  <li key={key} {...otherProps}>
                    <Avatar
                      alt={option.avatarUrl}
                      src={option.avatarUrl}
                      sx={{
                        mr: 1,
                        width: 24,
                        height: 24,
                        flexShrink: 0,
                      }}
                    />
                    {option.name}
                  </li>
                );
              }}
              renderValue={(selected: typeof _tourGuides, getItemProps) =>
                selected.map((option, index) => (
                  <Chip
                    {...getItemProps({ index })}
                    key={option.id}
                    label={option.name}
                    avatar={<Avatar alt={option.name} src={option.avatarUrl} />}
                    size="small"
                    variant="soft"
                  />
                ))
              }
            />
          </div>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Available</Typography>
            <Box sx={{ gap: 2, display: 'flex', flexDirection: { xs: 'column', md: 'row' } }}>
              <Field.DatePicker name="available.startDate" label="Start date" />
              <Field.DatePicker name="available.endDate" label="End date" />
            </Box>
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Duration</Typography>
            <Field.Text name="durations" placeholder="Ex: 2 days, 4 days 3 nights..." />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Destination</Typography>
            <Field.CountrySelect fullWidth name="destination" placeholder="+ Destination" />
          </Stack>

          <Stack spacing={1}>
            <Typography variant="subtitle2">Services</Typography>
            <Field.MultiCheckbox
              name="services"
              options={TOUR_SERVICE_OPTIONS}
              sx={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)' }}
            />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Tags</Typography>
            <Field.Autocomplete
              name="tags"
              placeholder="+ Tags"
              multiple
              freeSolo
              disableCloseOnSelect
              options={_tags.map((option) => option)}
              getOptionLabel={(option) => option}
              slotProps={{
                chip: { color: 'info' },
              }}
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
        control={
          <Switch
            defaultChecked
            slotProps={{
              input: { id: 'publish-switch' },
            }}
          />
        }
        sx={{ flexGrow: 1, pl: 3 }}
      />

      <Button type="submit" variant="contained" size="large" loading={isSubmitting} sx={{ ml: 2 }}>
        {!currentTour ? 'Create tour' : 'Save changes'}
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
