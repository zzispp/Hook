import type { StepperProps } from '@mui/material/Stepper';
import type { StepIconProps } from '@mui/material/StepIcon';

import { useState, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Step from '@mui/material/Step';
import Paper from '@mui/material/Paper';
import Button from '@mui/material/Button';
import Stepper from '@mui/material/Stepper';
import { styled } from '@mui/material/styles';
import StepLabel from '@mui/material/StepLabel';
import StepConnector, { stepConnectorClasses } from '@mui/material/StepConnector';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

const QontoConnector = styled(StepConnector)(({ theme }) => ({
  [`&.${stepConnectorClasses.alternativeLabel}`]: {
    top: 10,
    left: 'calc(-50% + 16px)',
    right: 'calc(50% + 16px)',
  },
  [`&.${stepConnectorClasses.active}`]: {
    [`& .${stepConnectorClasses.line}`]: { borderColor: theme.vars.palette.success.main },
  },
  [`&.${stepConnectorClasses.completed}`]: {
    [`& .${stepConnectorClasses.line}`]: { borderColor: theme.vars.palette.success.main },
  },
  [`& .${stepConnectorClasses.line}`]: {
    borderRadius: 1,
    borderTopWidth: 3,
    borderColor: theme.vars.palette.divider,
  },
}));

const QontoStepIconRoot = styled('div')<{
  ownerState: { active?: boolean };
}>(({ theme, ownerState }) => ({
  height: 22,
  display: 'flex',
  alignItems: 'center',
  color: theme.vars.palette.text.disabled,
  ...(ownerState.active && { color: theme.vars.palette.success.main }),
  '& .QontoStepIcon-completedIcon': {
    zIndex: 1,
    fontSize: 18,
    color: theme.vars.palette.success.main,
  },
  '& .QontoStepIcon-circle': {
    width: 8,
    height: 8,
    borderRadius: '50%',
    backgroundColor: 'currentColor',
  },
}));

const ColorlibConnector = styled(StepConnector)(({ theme }) => ({
  [`&.${stepConnectorClasses.alternativeLabel}`]: { top: 22 },
  [`&.${stepConnectorClasses.active}`]: {
    [`& .${stepConnectorClasses.line}`]: {
      backgroundImage: `linear-gradient(to top, ${theme.vars.palette.error.light}, ${theme.vars.palette.error.main})`,
    },
  },
  [`&.${stepConnectorClasses.completed}`]: {
    [`& .${stepConnectorClasses.line}`]: {
      backgroundImage: `linear-gradient(to top, ${theme.vars.palette.error.light}, ${theme.vars.palette.error.main})`,
    },
  },
  [`& .${stepConnectorClasses.line}`]: {
    height: 3,
    border: 0,
    borderRadius: 1,
    backgroundColor: theme.vars.palette.divider,
  },
}));

const ColorlibStepIconRoot = styled('div')<{
  ownerState: { completed?: boolean; active?: boolean };
}>(({ theme, ownerState }) => ({
  zIndex: 1,
  width: 50,
  height: 50,
  display: 'flex',
  borderRadius: '50%',
  alignItems: 'center',
  justifyContent: 'center',
  color: theme.vars.palette.text.disabled,
  backgroundColor: theme.vars.palette.grey[300],
  ...theme.applyStyles('dark', {
    backgroundColor: theme.vars.palette.grey[700],
  }),
  ...(ownerState.active && {
    color: theme.vars.palette.common.white,
    boxShadow: '0 4px 10px 0 rgba(0,0,0,0.25)',
    backgroundImage: `linear-gradient(to top, ${theme.vars.palette.error.light}, ${theme.vars.palette.error.main})`,
  }),
  ...(ownerState.completed && {
    color: theme.vars.palette.common.white,
    backgroundImage: `linear-gradient(to top, ${theme.vars.palette.error.light}, ${theme.vars.palette.error.main})`,
  }),
}));

function QontoStepIcon({ active, completed, className }: StepIconProps) {
  return (
    <QontoStepIconRoot ownerState={{ active }} className={className}>
      {completed ? (
        <Iconify width={24} icon="eva:checkmark-fill" className="QontoStepIcon-completedIcon" />
      ) : (
        <div className="QontoStepIcon-circle" />
      )}
    </QontoStepIconRoot>
  );
}

function ColorlibStepIcon({ active, completed, className, icon }: StepIconProps) {
  const icons: {
    [index: string]: React.ReactElement;
  } = {
    1: <Iconify icon="solar:settings-bold" width={24} />,
    2: <Iconify icon="solar:user-plus-bold" width={24} />,
    3: <Iconify icon="solar:monitor-bold" width={24} />,
  };

  return (
    <ColorlibStepIconRoot ownerState={{ completed, active }} className={className}>
      {icons[String(icon)]}
    </ColorlibStepIconRoot>
  );
}

// ----------------------------------------------------------------------

type Props = StepperProps & {
  dataSteps: {
    label: string;
    description?: string;
  }[];
};

export function CustomizedSteppers({ dataSteps, ...other }: Props) {
  const [activeStep, setActiveStep] = useState(0);

  const isCompleted = activeStep === dataSteps.length;
  const isFirstStep = activeStep === 0;
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

  const renderStepperDot = () => (
    <Stepper alternativeLabel activeStep={activeStep} connector={<QontoConnector />} {...other}>
      {dataSteps.map((step) => (
        <Step key={step.label}>
          <StepLabel slots={{ stepIcon: QontoStepIcon }}>{step.label}</StepLabel>
        </Step>
      ))}
    </Stepper>
  );

  const renderStepperIcon = () => (
    <Stepper
      alternativeLabel
      activeStep={activeStep}
      connector={<ColorlibConnector />}
      sx={{ mt: 5 }}
      {...other}
    >
      {dataSteps.map((step) => (
        <Step key={step.label}>
          <StepLabel slots={{ stepIcon: ColorlibStepIcon }}>{step.label}</StepLabel>
        </Step>
      ))}
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

          <Button variant="contained" onClick={handleNext}>
            {isLastStep ? 'Finish' : 'Next'}
          </Button>
        </>
      )}
    </Box>
  );

  return (
    <Box sx={{ width: 1 }}>
      {renderStepperDot()}
      {renderStepperIcon()}
      {renderContent(isCompleted ? 'All steps completed!' : dataSteps[activeStep].description)}
      {renderControls()}
    </Box>
  );
}
