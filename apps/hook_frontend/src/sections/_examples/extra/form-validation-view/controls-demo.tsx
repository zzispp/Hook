import type { ControlsSchemaType } from './schema';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Divider from '@mui/material/Divider';
import Backdrop from '@mui/material/Backdrop';
import CircularProgress from '@mui/material/CircularProgress';

import { Form, Field } from 'src/components/hook-form';

import { ControlsSchema } from './schema';
import { ComponentBox } from '../../layout';
import { FormGrid, FormActions } from './components';
import { ValuesPreview } from './components/values-preview';

// ----------------------------------------------------------------------

const defaultValues: ControlsSchemaType = {
  rating: 0,
  slider: 8,
  switch: false,
  radioGroup: '',
  checkbox: false,
  multiSwitch: [],
  multiCheckbox: [],
  sliderRange: [15, 80],
};

type Props = {
  debug: boolean;
  onClose: () => void;
};

export function ControlsDemo({ debug, onClose }: Props) {
  const methods = useForm({
    resolver: zodResolver(ControlsSchema),
    defaultValues,
  });

  const {
    reset,
    handleSubmit,
    formState: { isSubmitting, errors },
  } = methods;

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 3000));
      reset();
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

  const renderRating = () => <Field.Rating name="rating" />;

  const renderCheckbox = () => (
    <Box sx={{ width: 1 }}>
      <Field.Checkbox name="checkbox" label="RHFCheckbox" />
      <Divider sx={{ my: 3, width: 1 }} />
      <Field.MultiCheckbox
        name="multiCheckbox"
        label="RHFMultiCheckbox"
        options={[
          { label: 'Option 1', value: 'checkbox-1' },
          { label: 'Option 2', value: 'checkbox-2' },
          { label: 'Option 3', value: 'checkbox-3' },
        ]}
        sx={{ gap: 0.75 }}
      />
    </Box>
  );

  const renderSwitch = () => (
    <Box sx={{ width: 1 }}>
      <Field.Switch name="switch" label="RHFSwitch" />
      <Divider sx={{ my: 3, width: 1 }} />
      <Field.MultiSwitch
        name="multiSwitch"
        label="RHFMultiSwitch"
        options={[
          { label: 'Option 1', value: 'switch-1' },
          { label: 'Option 2', value: 'switch-2' },
          { label: 'Option 3', value: 'switch-3' },
        ]}
        sx={{ gap: 0.75 }}
      />
    </Box>
  );

  const renderRadio = () => (
    <Box sx={{ width: 1 }}>
      <Field.RadioGroup
        name="radioGroup"
        label="RHFRadioGroup"
        options={[
          { label: 'Option 1', value: 'radio-1' },
          { label: 'Option 2', value: 'radio-2' },
          { label: 'Option 3', value: 'radio-3' },
        ]}
        sx={{ gap: 0.75 }}
      />
    </Box>
  );

  const renderSlider = () => (
    <Box sx={{ gap: 5, width: 1, display: 'flex', flexDirection: 'column' }}>
      <Field.Slider name="slider" />
      <Field.Slider name="sliderRange" />
    </Box>
  );

  return (
    <>
      {isSubmitting && (
        <Backdrop open sx={[(theme) => ({ zIndex: theme.zIndex.modal + 1 })]}>
          <CircularProgress color="warning" />
        </Backdrop>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        {debug && <ValuesPreview onCloseDebug={onClose} />}

        <FormActions
          loading={isSubmitting}
          disabled={Object.keys(errors).length === 0}
          onReset={() => reset()}
        />

        <FormGrid>
          <ComponentBox title="Rating">{renderRating()}</ComponentBox>
          <ComponentBox title="Checkbox">{renderCheckbox()}</ComponentBox>
          <ComponentBox title="Switch">{renderSwitch()}</ComponentBox>
          <ComponentBox title="Radio">{renderRadio()}</ComponentBox>
          <ComponentBox title="Slider">{renderSlider()}</ComponentBox>
        </FormGrid>
      </Form>
    </>
  );
}
