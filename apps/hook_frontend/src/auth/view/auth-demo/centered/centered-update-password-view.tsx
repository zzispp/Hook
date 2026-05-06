'use client';

import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { paths } from 'src/routes/paths';

import { SentIcon } from 'src/assets/icons';

import { Iconify } from 'src/components/iconify';
import { Form, Field, schemaUtils } from 'src/components/hook-form';

import { FormHead } from '../../../components/form-head';
import { FormResendCode } from '../../../components/form-resend-code';
import { FormReturnLink } from '../../../components/form-return-link';

// ----------------------------------------------------------------------

export type UpdatePasswordSchemaType = z.infer<typeof UpdatePasswordSchema>;

export const UpdatePasswordSchema = z
  .object({
    code: z
      .string()
      .min(1, { error: 'Code is required!' })
      .min(6, { error: 'Code must be at least 6 characters!' }),
    email: schemaUtils.email(),
    password: z
      .string()
      .min(1, { error: 'Password is required!' })
      .min(6, { error: 'Password must be at least 6 characters!' }),
    confirmPassword: z.string().min(1, { error: 'Confirm password is required!' }),
  })
  .refine((val) => val.password === val.confirmPassword, {
    error: 'Passwords do not match!',
    path: ['confirmPassword'],
  });

// ----------------------------------------------------------------------

export function CenteredUpdatePasswordView() {
  const showPassword = useBoolean();

  const defaultValues: UpdatePasswordSchemaType = {
    code: '',
    email: '',
    password: '',
    confirmPassword: '',
  };

  const methods = useForm({
    resolver: zodResolver(UpdatePasswordSchema),
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
        name="email"
        label="Email address"
        placeholder="example@gmail.com"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Field.Code name="code" />

      <Field.Text
        name="password"
        label="Password"
        placeholder="6+ characters"
        type={showPassword.value ? 'text' : 'password'}
        slotProps={{
          inputLabel: { shrink: true },
          input: {
            endAdornment: (
              <InputAdornment position="end">
                <IconButton onClick={showPassword.onToggle} edge="end">
                  <Iconify icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />

      <Field.Text
        name="confirmPassword"
        label="Confirm new password"
        type={showPassword.value ? 'text' : 'password'}
        slotProps={{
          inputLabel: { shrink: true },
          input: {
            endAdornment: (
              <InputAdornment position="end">
                <IconButton onClick={showPassword.onToggle} edge="end">
                  <Iconify icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />

      <Button
        fullWidth
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator="Update password..."
      >
        Update password
      </Button>
    </Box>
  );

  return (
    <>
      <FormHead
        icon={<SentIcon />}
        title="Request sent successfully!"
        description={`We've sent a 6-digit confirmation email to your email. \nPlease enter the code in below box to verify your email.`}
      />

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>

      <FormResendCode onResendCode={() => {}} value={0} disabled={false} />

      <FormReturnLink href={paths.authDemo.centered.signIn} />
    </>
  );
}
