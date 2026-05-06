import type { OtherSchemaType } from './schema';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

import Backdrop from '@mui/material/Backdrop';
import CircularProgress from '@mui/material/CircularProgress';

import { Form, Field } from 'src/components/hook-form';

import { OtherSchema } from './schema';
import { ComponentBox } from '../../layout';
import { ValuesPreview } from './components/values-preview';
import { FormGrid, FormActions, FieldContainer } from './components';

// ----------------------------------------------------------------------

const defaultValues: OtherSchemaType = {
  editor: '',
  singleUpload: '',
  multiUpload: [],
};

type Props = {
  debug: boolean;
  onClose: () => void;
};

export function OtherDemo({ debug, onClose }: Props) {
  const methods = useForm({
    resolver: zodResolver(OtherSchema),
    defaultValues,
  });

  const {
    watch,
    reset,
    setValue,
    handleSubmit,
    formState: { isSubmitting, errors },
  } = methods;

  const values = watch();

  const onSubmit = handleSubmit(async (data) => {
    try {
      await new Promise((resolve) => setTimeout(resolve, 3000));
      reset();
      console.info('DATA', data);
    } catch (error) {
      console.error(error);
    }
  });

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

        <FormGrid sx={{ gridTemplateColumns: { xs: 'repeat(1, 1fr)', md: 'repeat(2, 1fr)' } }}>
          <ComponentBox title="Upload">
            <FieldContainer label="Single file upload">
              <Field.Upload
                name="singleUpload"
                maxSize={3145728}
                onDelete={() => setValue('singleUpload', null, { shouldValidate: true })}
              />
            </FieldContainer>

            <FieldContainer label="Multiple files upload">
              <Field.Upload
                multiple
                name="multiUpload"
                maxSize={3145728}
                onRemove={(inputFile) =>
                  setValue(
                    'multiUpload',
                    values.multiUpload.filter((file) => file !== inputFile),
                    { shouldValidate: true }
                  )
                }
                onRemoveAll={() => setValue('multiUpload', [], { shouldValidate: true })}
                onUpload={() => console.info('ON UPLOAD')}
              />
            </FieldContainer>
          </ComponentBox>

          <ComponentBox title="Editor" sx={{ display: 'block' }}>
            <Field.Editor fullItem name="editor" sx={{ maxHeight: 480 }} />
          </ComponentBox>
        </FormGrid>
      </Form>
    </>
  );
}
