'use client';

import Box from '@mui/material/Box';

import { CustomizedSteppers } from './customized-steppers';
import { MobileStepperVariant } from './mobile-stepper-variant';
import { VerticalLinearStepper } from './vertical-linear-stepper';
import { HorizontalLinearStepper } from './horizontal-linear-stepper';
import { ComponentBox, contentStyles, ComponentLayout } from '../../layout';

// ----------------------------------------------------------------------

const STEPS = [
  {
    label: 'Select campaign settings',
    description: 'Set budget, targeting, and display preferences for your campaign.',
  },
  {
    label: 'Create an ad group',
    description: 'Group related ads and keywords together for better targeting.',
  },
  {
    label: 'Create an ad',
    description: 'Write compelling ad copy, add images or videos, and define CTAs.',
  },
];

const DEMO_COMPONENTS = [
  {
    name: 'Horizontal linear stepper',
    component: (
      <ComponentBox>
        <HorizontalLinearStepper dataSteps={STEPS} />
      </ComponentBox>
    ),
  },
  {
    name: 'Linear alternative label',
    component: (
      <ComponentBox>
        <HorizontalLinearStepper dataSteps={STEPS} alternativeLabel />
      </ComponentBox>
    ),
  },
  {
    name: 'Vertical linear stepper',
    component: (
      <ComponentBox>
        <VerticalLinearStepper dataSteps={STEPS} />
      </ComponentBox>
    ),
  },
  {
    name: 'Customized stepper',
    component: (
      <ComponentBox>
        <CustomizedSteppers dataSteps={STEPS} />
      </ComponentBox>
    ),
  },
  {
    name: 'Mobile stepper',
    component: (
      <Box sx={contentStyles.grid()}>
        <ComponentBox title="Text">
          <MobileStepperVariant dataSteps={STEPS} variant="text" />
        </ComponentBox>

        <ComponentBox title="Dots">
          <MobileStepperVariant dataSteps={STEPS} variant="dots" />
        </ComponentBox>

        <ComponentBox title="Progress">
          <MobileStepperVariant dataSteps={STEPS} variant="progress" />
        </ComponentBox>
      </Box>
    ),
  },
];

// ----------------------------------------------------------------------

export function StepperView() {
  return (
    <ComponentLayout
      sectionData={DEMO_COMPONENTS}
      heroProps={{
        heading: 'Stepper',
        moreLinks: ['https://mui.com/material-ui/react-stepper/'],
      }}
    />
  );
}
