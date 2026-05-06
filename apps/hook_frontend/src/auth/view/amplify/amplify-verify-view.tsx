'use client';

import * as z from 'zod';
import { useCallback } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useCountdownSeconds } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';
import { useRouter, useSearchParams } from 'src/routes/hooks';

import { EmailInboxIcon } from 'src/assets/icons';

import { Form, Field, schemaUtils } from 'src/components/hook-form';

import { FormHead } from '../../components/form-head';
import { FormReturnLink } from '../../components/form-return-link';
import { FormResendCode } from '../../components/form-resend-code';
import { confirmSignUp, resendSignUpCode } from '../../context/amplify';

// ----------------------------------------------------------------------

export type VerifySchemaType = z.infer<typeof VerifySchema>;

export const VerifySchema = z.object({
  code: z
    .string()
    .min(1, { error: 'Code is required!' })
    .min(6, { error: 'Code must be at least 6 characters!' }),
  email: schemaUtils.email(),
});

// ----------------------------------------------------------------------

export function AmplifyVerifyView() {
  const router = useRouter();

  const searchParams = useSearchParams();

  const email = searchParams.get('email');

  const countdown = useCountdownSeconds(5);

  const defaultValues: VerifySchemaType = {
    code: '',
    email: email || '',
  };

  const methods = useForm({
    resolver: zodResolver(VerifySchema),
    defaultValues,
  });

  const {
    watch,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const values = watch();

  const onSubmit = handleSubmit(async (data) => {
    try {
      await confirmSignUp({ username: data.email, confirmationCode: data.code });
      router.push(paths.auth.amplify.signIn);
    } catch (error) {
      console.error(error);
    }
  });

  const handleResendCode = useCallback(async () => {
    if (!countdown.isCounting) {
      try {
        countdown.reset();
        countdown.start();

        await resendSignUpCode?.({ username: values.email });
      } catch (error) {
        console.error(error);
      }
    }
  }, [countdown, values.email]);

  const renderForm = () => (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="email"
        label="Email address"
        placeholder="example@gmail.com"
        slotProps={{ inputLabel: { shrink: true } }}
        disabled
      />

      <Field.Code name="code" />

      <Button
        fullWidth
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator="Verify..."
      >
        Verify
      </Button>
    </Box>
  );

  return (
    <>
      <FormHead
        icon={<EmailInboxIcon />}
        title="Please check your email!"
        description={`We've emailed a 6-digit confirmation code. \nPlease enter the code in the box below to verify your email.`}
      />

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>

      <FormResendCode
        onResendCode={handleResendCode}
        value={countdown.value}
        disabled={countdown.isCounting}
      />

      <FormReturnLink href={paths.auth.amplify.signIn} />
    </>
  );
}
