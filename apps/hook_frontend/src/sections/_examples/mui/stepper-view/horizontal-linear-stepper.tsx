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

// ----------------------------------------------------------------------

const isStepOptional = (step: number): boolean => step === 1;
const isStepSkipped = (step: number, skipped: Set<number>): boolean => skipped.has(step);

type Props = StepperProps & {
  dataSteps: {
    label: string;
    description?: string;
  }[];
};

export function HorizontalLinearStepper({ dataSteps, ...other }: Props) {
  const [activeStep, setActiveStep] = useState(0);
  const [skipped, setSkipped] = useState(new Set<number>());

  const isCompleted = activeStep === dataSteps.length;
  const isFirstStep = activeStep === 0;
  const isLastStep = activeStep === dataSteps.length - 1;

  const handleNext = useCallback(() => {
    let newSkipped = skipped;
    if (isStepSkipped(activeStep, skipped)) {
      newSkipped = new Set(newSkipped.values());
      newSkipped.delete(activeStep);
    }
    setSkipped(newSkipped);
    setActiveStep((prev) => prev + 1);
  }, [activeStep, skipped]);

  const handleBack = useCallback(() => {
    setActiveStep((prev) => prev - 1);
  }, []);

  const handleSkip = useCallback(() => {
    if (!isStepOptional(activeStep)) {
      // You probably want to guard against something like this,
      // it should never occur unless someone's actively trying to break something.
      throw new Error("You can't skip a step that isn't optional.");
    }
    setSkipped((prevSkipped) => {
      const newSkipped = new Set(prevSkipped.values());
      newSkipped.add(activeStep);
      return newSkipped;
    });
    setActiveStep((prev) => prev + 1);
  }, [activeStep]);

  const handleReset = useCallback(() => {
    setActiveStep(0);
  }, []);

  const renderStepper = () => (
    <Stepper activeStep={activeStep} {...other}>
      {dataSteps.map((step, index) => {
        const stepProps: { completed?: boolean } = {};
        const labelProps: { optional?: React.ReactNode } = {};
        if (isStepOptional(index)) {
          labelProps.optional = <Typography variant="caption">Optional</Typography>;
        }
        if (isStepSkipped(index, skipped)) {
          stepProps.completed = false;
        }
        return (
          <Step key={step.label} {...stepProps}>
            <StepLabel {...labelProps}>{step.label}</StepLabel>
          </Step>
        );
      })}
    </Stepper>
  );

  const renderContent = (text?: string) => (
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
      {text}
    </Paper>
  );

  const renderControls = () => (
    <Box sx={{ mt: 3, gap: 1, display: 'flex', justifyContent: 'flex-end' }}>
      {isCompleted ? (
        <Button onClick={handleReset}>Reset</Button>
      ) : (
        <>
          <Button color="inherit" disabled={!isFirstStep} onClick={handleBack}>
            Back
          </Button>

          <Box component="span" sx={{ flexGrow: 1 }} />

          {isStepOptional(activeStep) && (
            <Button color="inherit" onClick={handleSkip}>
              Skip
            </Button>
          )}

          <Button variant="contained" onClick={handleNext}>
            {isLastStep ? 'Finish' : 'Next'}
          </Button>
        </>
      )}
    </Box>
  );

  return (
    <Box sx={{ width: 1 }}>
      {renderStepper()}
      {renderContent(isCompleted ? 'All steps completed!' : dataSteps[activeStep].description)}
      {renderControls()}
    </Box>
  );
}
