import type { ISocialLink } from 'src/types/common';

import { useForm } from 'react-hook-form';

import Card from '@mui/material/Card';
import Button from '@mui/material/Button';
import InputAdornment from '@mui/material/InputAdornment';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';
import { Form, Field } from 'src/components/hook-form';

// ----------------------------------------------------------------------

type Props = {
  socialLinks: ISocialLink;
};

export function AccountSocials({ socialLinks }: Props) {
  const defaultValues: ISocialLink = {
    facebook: '',
    instagram: '',
    linkedin: '',
    twitter: '',
  };

  const methods = useForm({
    defaultValues,
    values: socialLinks,
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      toast.success('Update success!');
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  return (
    <Form methods={methods} onSubmit={onSubmit}>
      <Card
        sx={{
          p: 3,
          gap: 3,
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {Object.keys(socialLinks).map((social) => (
          <Field.Text
            key={social}
            name={social}
            slotProps={{
              input: {
                startAdornment: (
                  <InputAdornment position="start">
                    {social === 'twitter' && <Iconify width={24} icon="socials:twitter" />}
                    {social === 'facebook' && <Iconify width={24} icon="socials:facebook" />}
                    {social === 'instagram' && <Iconify width={24} icon="socials:instagram" />}
                    {social === 'linkedin' && <Iconify width={24} icon="socials:linkedin" />}
                  </InputAdornment>
                ),
              },
            }}
          />
        ))}

        <Button type="submit" variant="contained" loading={isSubmitting} sx={{ ml: 'auto' }}>
          Save changes
        </Button>
      </Card>
    </Form>
  );
}
