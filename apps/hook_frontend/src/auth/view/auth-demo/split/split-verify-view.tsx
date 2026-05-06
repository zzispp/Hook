'use client';

import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';

import { paths } from 'src/routes/paths';

import { EmailInboxIcon } from 'src/assets/icons';

import { Form, Field, schemaUtils } from 'src/components/hook-form';

import { FormHead } from '../../../components/form-head';
import { FormReturnLink } from '../../../components/form-return-link';
import { FormResendCode } from '../../../components/form-resend-code';

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

export function SplitVerifyView() {
  const defaultValues: VerifySchemaType = {
    code: '',
    email: '',
  };

  const methods = useForm({
    resolver: zodResolver(VerifySchema),
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

      <FormResendCode onResendCode={() => {}} value={0} disabled={false} />

      <FormReturnLink href={paths.authDemo.split.signIn} />
    </>
  );
}
