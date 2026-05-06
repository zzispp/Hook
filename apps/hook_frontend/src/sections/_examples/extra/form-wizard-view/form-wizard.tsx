import * as z from 'zod';
import { useForm } from 'react-hook-form';
import { useState, useCallback } from 'react';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Button from '@mui/material/Button';

import { Form, schemaUtils } from 'src/components/hook-form';

import { Stepper, StepOne, StepTwo, StepThree, StepCompleted } from './form-steps';

// ----------------------------------------------------------------------

const STEPS = ['Step one', 'Step two', 'Step three'];

const StepOneSchema = z.object({
  firstName: z.string().min(1, { error: 'Full name is required!' }),
  lastName: z.string().min(1, { error: 'Last name is required!' }),
});

const StepTwoSchema = z.object({
  age: schemaUtils.nullableInput(
    z.coerce
      .number()
      .int()
      .min(1, { error: 'Age is required!' })
      .min(18, { error: 'Age must be between 18 and 80' })
      .max(80, { error: 'Age must be between 18 and 80' }),
    { error: 'Age is required!' }
  ),
});

const StepThreeSchema = z.object({
  email: schemaUtils.email(),
});

const WizardSchema = z.object({
  stepOne: StepOneSchema,
  stepTwo: StepTwoSchema,
  stepThree: StepThreeSchema,
});

type WizardSchemaType = z.infer<typeof WizardSchema>;

// ----------------------------------------------------------------------

const defaultValues: WizardSchemaType = {
  stepOne: { firstName: '', lastName: '' },
  stepTwo: { age: null },
  stepThree: { email: '' },
};

export function FormWizard() {
  const [activeStep, setActiveStep] = useState(0);

  const methods = useForm({
    mode: 'onChange',
    resolver: zodResolver(WizardSchema),
    defaultValues,
  });

  const {
    reset,
    trigger,
    clearErrors,
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const handleNext = useCallback(
    async (step?: 'stepOne' | 'stepTwo') => {
      if (step) {
        const isValid = await trigger(step);

        if (isValid) {
          clearErrors();
          setActiveStep((currentStep) => currentStep + 1);
        }
      } else {
        setActiveStep((currentStep) => currentStep + 1);
      }
    },
    [trigger, clearErrors]
  );

  const handleBack = useCallback(() => {
    setActiveStep((currentStep) => currentStep - 1);
  }, []);

  const handleReset = useCallback(() => {
    reset();
    setActiveStep(0);
  }, [reset]);

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 1000));

      console.info('DATA', data);
      handleNext();
    } catch (error) {
      console.error(error);
    }
  });

  const completedStep = activeStep === STEPS.length;

  return (
    <Card
      sx={{
        p: 5,
        width: 1,
        mx: 'auto',
        maxWidth: 720,
      }}
    >
      <Stepper steps={STEPS} activeStep={activeStep} />

      <Form methods={methods} onSubmit={onSubmit}>
        <Box
          sx={[
            (theme) => ({
              p: 3,
              mb: 3,
              gap: 3,
              minHeight: 240,
              display: 'flex',
              borderRadius: 1.5,
              flexDirection: 'column',
              border: `dashed 1px ${theme.vars.palette.divider}`,
            }),
          ]}
        >
          {activeStep === 0 && <StepOne />}
          {activeStep === 1 && <StepTwo />}
          {activeStep === 2 && <StepThree />}
          {completedStep && <StepCompleted onReset={handleReset} />}
        </Box>

        {!completedStep && (
          <Box sx={{ display: 'flex' }}>
            {activeStep !== 0 && <Button onClick={handleBack}>Back</Button>}

            <Box sx={{ flex: '1 1 auto' }} />

            {activeStep === 0 && (
              <Button type="submit" variant="contained" onClick={() => handleNext('stepOne')}>
                Next
              </Button>
            )}

            {activeStep === 1 && (
              <Button type="submit" variant="contained" onClick={() => handleNext('stepTwo')}>
                Next
              </Button>
            )}

            {activeStep === STEPS.length - 1 && (
              <Button type="submit" variant="contained" loading={isSubmitting}>
                Save changes
              </Button>
            )}
          </Box>
        )}
      </Form>
    </Card>
  );
}
