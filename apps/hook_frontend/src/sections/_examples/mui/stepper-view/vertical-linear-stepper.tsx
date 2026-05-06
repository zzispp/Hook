import type { StepperProps } from '@mui/material/Stepper';

import { useState, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Step from '@mui/material/Step';
import Paper from '@mui/material/Paper';
import Button from '@mui/material/Button';
import Stepper from '@mui/material/Stepper';
import StepLabel from '@mui/material/StepLabel';
import Typography from '@mui/material/Typography';
import StepContent from '@mui/material/StepContent';

// ----------------------------------------------------------------------

type Props = StepperProps & {
  dataSteps: {
    label: string;
    description?: string;
  }[];
};

export function VerticalLinearStepper({ dataSteps, ...other }: Props) {
  const [activeStep, setActiveStep] = useState(0);

  const isCompleted = activeStep === dataSteps.length;
  const isLastStep = activeStep === dataSteps.length - 1;

  const handleNext = useCallback(() => {
    setActiveStep((prev) => prev + 1);
  }, []);

  const handleBack = useCallback(() => {
    setActiveStep((prev) => prev - 1);
  }, []);

  const handleReset = useCallback(() => {
    setActiveStep(0);
  }, []);

  const renderStepper = () => (
    <Stepper activeStep={activeStep} orientation="vertical" {...other}>
      {dataSteps.map((step, index) => (
        <Step key={step.label}>
          <StepLabel
            optional={index === 2 ? <Typography variant="caption">Last step</Typography> : null}
          >
            {step.label}
          </StepLabel>
          <StepContent>
            <Typography>{step.description}</Typography>
            <Box sx={{ mt: 3 }}>
              <Button variant="contained" onClick={handleNext}>
                {isLastStep ? 'Finish' : 'Continue'}
              </Button>
              <Button disabled={index === 0} onClick={handleBack}>
                Back
              </Button>
            </Box>
          </StepContent>
        </Step>
      ))}
    </Stepper>
  );

  const renderResetContent = () => (
    <Paper
      sx={[
        (theme) => ({
          p: 3,
          mt: 5,
          minHeight: 120,
          bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.12),
        }),
      ]}
    >
      <Typography sx={{ mb: 2 }}>All steps completed!</Typography>
      <Button onClick={handleReset}>Reset</Button>
    </Paper>
  );

  return (
    <Box sx={{ width: 1 }}>
      {renderStepper()}
      {isCompleted && renderResetContent()}
    </Box>
  );
}
