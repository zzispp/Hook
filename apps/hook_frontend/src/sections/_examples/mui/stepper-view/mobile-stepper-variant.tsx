import type { MobileStepperProps } from '@mui/material/MobileStepper';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Paper from '@mui/material/Paper';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import MobileStepper from '@mui/material/MobileStepper';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

export type Props = Omit<MobileStepperProps, 'steps' | 'backButton' | 'nextButton'> & {
  dataSteps: {
    label: string;
    description?: string;
  }[];
};

export function MobileStepperVariant({ dataSteps, ...other }: Props) {
  const theme = useTheme();
  const [activeStep, setActiveStep] = useState(0);

  const maxSteps = dataSteps.length;

  const handleNext = useCallback(() => {
    setActiveStep((prev) => prev + 1);
  }, []);

  const handleBack = useCallback(() => {
    setActiveStep((prev) => prev - 1);
  }, []);

  const renderLeftIcon = () => <Iconify icon="eva:arrow-ios-back-fill" />;
  const renderRightIcon = () => <Iconify icon="eva:arrow-ios-forward-fill" />;

  return (
    <Paper variant="outlined">
      <Box sx={{ p: 3, minHeight: 200 }}>
        <Typography variant="subtitle1" sx={{ mb: 1 }}>
          {dataSteps[activeStep].label}
        </Typography>
        <Typography variant="body2" sx={{ color: 'text.secondary' }}>
          {dataSteps[activeStep].description}
        </Typography>
      </Box>

      <Divider />

      <MobileStepper
        position="static"
        activeStep={activeStep}
        steps={dataSteps.length}
        nextButton={
          <Button size="small" onClick={handleNext} disabled={activeStep === maxSteps - 1}>
            Next
            {theme.direction === 'rtl' ? renderLeftIcon() : renderRightIcon()}
          </Button>
        }
        backButton={
          <Button size="small" onClick={handleBack} disabled={activeStep === 0}>
            {theme.direction === 'rtl' ? renderRightIcon() : renderLeftIcon()}
            Back
          </Button>
        }
        {...other}
      />
    </Paper>
  );
}
