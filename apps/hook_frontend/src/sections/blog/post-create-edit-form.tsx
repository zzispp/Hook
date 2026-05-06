import type { IPostItem } from 'src/types/blog';

import * as z from 'zod';
import { useCallback } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import Collapse from '@mui/material/Collapse';
import IconButton from '@mui/material/IconButton';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { paths } from 'src/routes/paths';
import { useRouter } from 'src/routes/hooks';

import { _tags } from 'src/_mock';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Form, Field, schemaUtils } from 'src/components/hook-form';

import { PostDetailsPreview } from './post-details-preview';

// ----------------------------------------------------------------------

export type PostCreateSchemaType = z.infer<typeof PostCreateSchema>;

export const PostCreateSchema = z.object({
  title: z.string().min(1, { error: 'Title is required!' }),
  description: z.string().min(1, { error: 'Description is required!' }),
  content: schemaUtils.editor().min(100, { error: 'Content must be at least 100 characters' }),
  coverUrl: schemaUtils.file({ error: 'Cover is required!' }),
  tags: z.string().array().min(2, { error: 'Must have at least 2 items!' }),
  metaKeywords: z.string().array().min(1, { error: 'Meta keywords is required!' }),
  // Not required
  metaTitle: z.string(),
  metaDescription: z.string(),
});

// ----------------------------------------------------------------------

type Props = {
  currentPost?: IPostItem;
};

export function PostCreateEditForm({ currentPost }: Props) {
  const router = useRouter();

  const showPreview = useBoolean();
  const openDetails = useBoolean(true);
  const openProperties = useBoolean(true);

  const defaultValues: PostCreateSchemaType = {
    title: '',
    description: '',
    content: '',
    coverUrl: null,
    tags: [],
    metaKeywords: [],
    metaTitle: '',
    metaDescription: '',
  };

  const methods = useForm({
    mode: 'all',
    resolver: zodResolver(PostCreateSchema),
    defaultValues,
    values: currentPost,
  });

  const {
    reset,
    watch,
    setValue,
    handleSubmit,
    formState: { isSubmitting, isValid },
  } = methods;

  const values = watch();

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      showPreview.onFalse();
      toast.success(currentPost ? 'Update success!' : 'Create success!');
      router.push(paths.dashboard.post.root);
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  const handleRemoveFile = useCallback(() => {
    setValue('coverUrl', null);
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
          <Field.Text name="title" label="Post title" />

          <Field.Text name="description" label="Description" multiline rows={3} />

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Content</Typography>
            <Field.Editor name="content" sx={{ maxHeight: 480 }} />
          </Stack>

          <Stack spacing={1.5}>
            <Typography variant="subtitle2">Cover</Typography>
            <Field.Upload name="coverUrl" maxSize={3145728} onDelete={handleRemoveFile} />
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
          <Field.Autocomplete
            name="tags"
            label="Tags"
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

          <Field.Text name="metaTitle" label="Meta title" />

          <Field.Text
            name="metaDescription"
            label="Meta description"
            fullWidth
            multiline
            rows={3}
          />

          <Field.Autocomplete
            name="metaKeywords"
            label="Meta keywords"
            placeholder="+ Keywords"
            multiple
            freeSolo
            disableCloseOnSelect
            options={_tags.map((option) => option)}
            getOptionLabel={(option) => option}
            slotProps={{
              chip: { color: 'info' },
            }}
          />

          <FormControlLabel
            label="Enable comments"
            control={<Switch defaultChecked slotProps={{ input: { id: 'comments-switch' } }} />}
          />
        </Stack>
      </Collapse>
    </Card>
  );

  const renderActions = () => (
    <Box
      sx={{
        display: 'flex',
        flexWrap: 'wrap',
        alignItems: 'center',
        justifyContent: 'flex-end',
      }}
    >
      <FormControlLabel
        label="Publish"
        control={<Switch defaultChecked slotProps={{ input: { id: 'publish-switch' } }} />}
        sx={{ pl: 3, flexGrow: 1 }}
      />

      <div>
        <Button color="inherit" variant="outlined" size="large" onClick={showPreview.onTrue}>
          Preview
        </Button>

        <Button
          type="submit"
          variant="contained"
          size="large"
          loading={isSubmitting}
          sx={{ ml: 2 }}
        >
          {!currentPost ? 'Create post' : 'Save changes'}
        </Button>
      </div>
    </Box>
  );

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Stack spacing={5} sx={{ mx: 'auto', maxWidth: { xs: 720, xl: 880 } }}>
        {renderDetails()}
        {renderProperties()}
        {renderActions()}
      </Stack>

      <PostDetailsPreview
        isValid={isValid}
        onSubmit={onSubmit}
        title={values.title}
        open={showPreview.value}
        content={values.content}
        onClose={showPreview.onFalse}
        coverUrl={values.coverUrl}
        isSubmitting={isSubmitting}
        description={values.description}
      />
    </Form>
  );
}
