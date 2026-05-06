import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

// ----------------------------------------------------------------------

export type CommentSchemaType = z.infer<typeof CommentSchema>;

export const CommentSchema = z.object({
  comment: z.string().min(1, { error: 'Comment is required!' }),
});

// ----------------------------------------------------------------------

export function PostCommentForm() {
  const defaultValues: CommentSchemaType = {
    comment: '',
  };

  const methods = useForm({
    resolver: zodResolver(CommentSchema),
    defaultValues,
  });

  const {
    reset,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      reset();
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
        <Field.Text
          name="comment"
          placeholder="Write some of your comments..."
          multiline
          rows={4}
        />

        <Box sx={{ display: 'flex', alignItems: 'center' }}>
          <Box sx={{ flexGrow: 1, display: 'flex', alignItems: 'center' }}>
            <IconButton>
              <Iconify icon="solar:gallery-add-bold" />
            </IconButton>

            <IconButton>
              <Iconify icon="eva:attach-2-fill" />
            </IconButton>

            <IconButton>
              <Iconify icon="eva:smiling-face-fill" />
            </IconButton>
          </Box>

          <Button type="submit" variant="contained" loading={isSubmitting}>
            Post comment
          </Button>
        </Box>
      </Box>
    </Form>
  );
}
