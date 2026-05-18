import type { ButtonProps } from '@mui/material/Button';

import Button from '@mui/material/Button';

import { RouterLink } from 'src/routes/components';

import { CONFIG } from 'src/global-config';
import { useTranslate } from 'src/locales';

// ----------------------------------------------------------------------

export function SignInButton({ sx, ...other }: ButtonProps) {
  const { t } = useTranslate('common');

  return (
    <Button
      component={RouterLink}
      href={CONFIG.auth.redirectPath}
      variant="outlined"
      sx={sx}
      {...other}
    >
      {t('nav.signIn')}
    </Button>
  );
}
