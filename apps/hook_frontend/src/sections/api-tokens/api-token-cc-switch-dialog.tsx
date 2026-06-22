'use client';

import type { CcSwitchFormat } from './api-token-cc-switch-utils';
import type { CcSwitchImportDialogState } from './api-token-cc-switch-state';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Alert from '@mui/material/Alert';
import Radio from '@mui/material/Radio';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import LinearProgress from '@mui/material/LinearProgress';
import ListItemButton from '@mui/material/ListItemButton';

import { useTranslate } from 'src/locales/use-locales';

import { CC_SWITCH_FORMATS } from './api-token-cc-switch-utils';

export function ApiTokenCcSwitchDialog({
  state,
}: {
  state: CcSwitchImportDialogState;
}) {
  const { t } = useTranslate('admin');

  return (
    <Dialog fullWidth maxWidth="sm" open={state.open} onClose={state.closeDialog}>
      <DialogTitle>{t('dialogs.importCcSwitch')}</DialogTitle>
      <DialogContent dividers>
        <Stack spacing={2}>
          <Alert severity="info">{t('tokens.ccSwitch.usageSummary')}</Alert>
          <Stack spacing={0.5}>
            <Typography variant="subtitle2">
              {state.targetToken?.name ?? t('common.loading')}
            </Typography>
            <Typography variant="body2" color="text.secondary">
              {stepLabel(state.step, t)}
            </Typography>
          </Stack>
          {state.loading ? <LinearProgress /> : null}
          {state.error ? (
            <Alert
              severity="error"
              action={
                <Button color="inherit" size="small" onClick={() => void state.reload()}>
                  {t('tokens.ccSwitch.retry')}
                </Button>
              }
            >
              {state.error}
            </Alert>
          ) : null}
          <CurrentStepContent state={state} />
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={state.closeDialog}>
          {t('common.cancel')}
        </Button>
        {state.step === 'format' ? (
          <Button variant="outlined" onClick={state.backToEndpoint}>
            {t('tokens.ccSwitch.backToEndpoint')}
          </Button>
        ) : null}
        {state.step === 'model' ? (
          <Button variant="outlined" onClick={state.backToFormat}>
            {t('tokens.ccSwitch.back')}
          </Button>
        ) : null}
        <Button
          variant="contained"
          disabled={actionDisabled(state)}
          onClick={() => void handlePrimaryAction(state)}
        >
          {primaryActionLabel(state.step, t)}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function CurrentStepContent({ state }: { state: CcSwitchImportDialogState }) {
  if (state.step === 'endpoint') {
    return <EndpointList state={state} />;
  }
  if (state.step === 'format') {
    return <FormatList state={state} />;
  }
  return <ModelList state={state} />;
}

function EndpointList({ state }: { state: CcSwitchImportDialogState }) {
  const { t } = useTranslate('admin');

  if (state.apiEndpoints.length === 0) {
    return <Alert severity="warning">{t('tokens.ccSwitch.noApiEndpoints')}</Alert>;
  }

  return (
    <List disablePadding sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}` }}>
      {state.apiEndpoints.map((endpoint, index) => (
        <Box key={endpoint.id}>
          {index > 0 ? <Divider /> : null}
          <ListItemButton
            selected={state.selectedEndpointId === endpoint.id}
            onClick={() => state.setSelectedEndpointId(endpoint.id)}
          >
            <Radio checked={state.selectedEndpointId === endpoint.id} tabIndex={-1} />
            <ListItemText
              primary={
                <Stack direction="row" spacing={1} alignItems="center">
                  <Typography variant="subtitle2">{endpoint.name}</Typography>
                  {endpoint.isDefault ? (
                    <Typography variant="caption" color="info.main">
                      {t('tokens.apiEndpoints.defaultBadge')}
                    </Typography>
                  ) : null}
                </Stack>
              }
              secondary={endpoint.url}
            />
          </ListItemButton>
        </Box>
      ))}
    </List>
  );
}

function FormatList({ state }: { state: CcSwitchImportDialogState }) {
  const { t } = useTranslate('admin');

  return (
    <List disablePadding sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}` }}>
      {CC_SWITCH_FORMATS.map((format, index) => {
        const count = countForFormat(state, format);
        const disabled = count === 0;

        return (
          <Box key={format}>
            {index > 0 ? <Divider /> : null}
            <ListItemButton
              disabled={disabled || state.loading}
              selected={state.selectedFormat === format}
              onClick={() => state.setSelectedFormat(format)}
            >
              <Radio checked={state.selectedFormat === format} disabled={disabled} tabIndex={-1} />
              <ListItemText
                primary={formatLabel(format, t)}
                secondary={t('tokens.ccSwitch.modelCount', { count })}
              />
            </ListItemButton>
          </Box>
        );
      })}
    </List>
  );
}

function ModelList({ state }: { state: CcSwitchImportDialogState }) {
  const { t } = useTranslate('admin');

  if (state.selectedFormatModels.length === 0) {
    return <Alert severity="warning">{t('tokens.ccSwitch.noModelsForFormat')}</Alert>;
  }

  return (
    <List disablePadding sx={{ border: (theme) => `1px solid ${theme.vars.palette.divider}` }}>
      {state.selectedFormatModels.map((model, index) => (
        <Box key={model.id}>
          {index > 0 ? <Divider /> : null}
          <ListItemButton
            selected={state.selectedModelId === model.id}
            onClick={() => state.setSelectedModelId(model.id)}
          >
            <Radio checked={state.selectedModelId === model.id} tabIndex={-1} />
            <ListItemText primary={model.label} secondary={model.id} />
          </ListItemButton>
        </Box>
      ))}
    </List>
  );
}

function countForFormat(state: CcSwitchImportDialogState, format: CcSwitchFormat) {
  return state.modelOptions.filter((model) => model.format === format).length;
}

function formatLabel(
  format: CcSwitchFormat,
  t: (key: string, options?: Record<string, unknown>) => string
) {
  return t(`tokens.ccSwitch.formats.${format}`);
}

function stepLabel(
  step: 'endpoint' | 'format' | 'model',
  t: (key: string, options?: Record<string, unknown>) => string
) {
  if (step === 'endpoint') {
    return t('tokens.ccSwitch.selectEndpoint');
  }
  return step === 'format' ? t('tokens.ccSwitch.selectFormat') : t('tokens.ccSwitch.selectModel');
}

function actionDisabled(state: CcSwitchImportDialogState) {
  if (state.step === 'endpoint') {
    return !state.canContinueToFormat;
  }
  if (state.step === 'format') {
    return !state.canContinueToModel;
  }
  return !state.canImport;
}

function handlePrimaryAction(state: CcSwitchImportDialogState) {
  if (state.step === 'endpoint') {
    return state.goToFormatStep();
  }
  if (state.step === 'format') {
    return state.goToModelStep();
  }
  return state.importToCcSwitch();
}

function primaryActionLabel(
  step: 'endpoint' | 'format' | 'model',
  t: (key: string, options?: Record<string, unknown>) => string
) {
  return step === 'model' ? t('tokens.ccSwitch.openInCcSwitch') : t('tokens.ccSwitch.continue');
}
