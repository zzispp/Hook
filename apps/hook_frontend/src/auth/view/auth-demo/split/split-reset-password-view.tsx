'use client';

import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';

import { PasswordIcon } from 'src/assets/icons';

import { Form, Field, schemaUtils } from 'src/components/hook-form';

import { FormHead } from '../../../components/form-head';
import { FormReturnLink } from '../../../components/form-return-link';

// ----------------------------------------------------------------------

export type ResetPasswordSchemaType = z.infer<typeof ResetPasswordSchema>;

export const ResetPasswordSchema = z.object({
  email: schemaUtils.email(),
});

// ----------------------------------------------------------------------

export function SplitResetPasswordView() {
  const defaultValues: ResetPasswordSchemaType = {
    email: '',
  };

  const methods = useForm({
    resolver: zodResolver(ResetPasswordSchema),
    defaultValues,
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  const renderForm = () => (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        autoFocus
        name="email"
        label="Email address"
        placeholder="example@gmail.com"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Button
        fullWidth
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator="Send request..."
      >
        Send request
      </Button>
    </Box>
  );

  return (
    <>
      <FormHead
        icon={<PasswordIcon />}
        title="Forgot your password?"
        description={`Please enter the email address associated with your account and we'll email you a link to reset your password.`}
      />

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>

      <FormReturnLink href={paths.authDemo.split.signIn} />
    </>
  );
}
