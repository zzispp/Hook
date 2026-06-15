import type { GlobalModelForm } from './model-management-utils';

import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { useRoutingProfiles } from 'src/actions/routing';

import { GlobalModelFormFields } from './global-model-form-fields';

// ----------------------------------------------------------------------

type Props = {
  open: boolean;
  title: string;
  form: GlobalModelForm;
  isEdit: boolean;
  submitting: boolean;
  picker?: React.ReactNode;
  onClose: () => void;
  onSubmit: () => void;
  onChange: (form: GlobalModelForm) => void;
};

export function GlobalModelFormDialog({
  open,
  title,
  form,
  isEdit,
  submitting,
  picker,
  onClose,
  onSubmit,
  onChange,
}: Props) {
  const { t } = useTranslate('admin');
  const profiles = useRoutingProfiles();

  return (
    <Dialog fullWidth maxWidth="lg" open={open} onClose={onClose}>
      <DialogTitle>{title}</DialogTitle>
      <DialogContent>
        <Stack
          sx={{
            pt: 1,
            gap: 3,
            display: 'grid',
            gridTemplateColumns: { xs: '1fr', md: picker ? '320px 1fr' : '1fr' },
          }}
        >
          {picker}
          <GlobalModelFormFields
            form={form}
            isEdit={isEdit}
            profiles={profiles.items}
            onChange={onChange}
          />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
