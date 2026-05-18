import type { Theme } from '@mui/material/styles';

export const endpointGridSx = {
  display: 'grid',
  gap: 1,
  gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
};

export const endpointButtonSx = {
  gap: 1.5,
  p: 1.5,
  minHeight: 76,
  justifyContent: 'space-between',
  alignItems: 'center',
};

export const editorGridSx = {
  display: 'grid',
  gap: 2,
  gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
};

export const editorInputSx = {
  alignItems: 'flex-start',
  fontFamily: 'monospace',
  typography: 'caption',
};

export function resultSx(success: boolean) {
  return {
    p: 2,
    borderRadius: 1,
    border: (theme: Theme) => `1px solid ${success ? theme.vars.palette.success.main : theme.vars.palette.error.main}`,
    bgcolor: success ? 'success.lighter' : 'error.lighter',
  };
}
