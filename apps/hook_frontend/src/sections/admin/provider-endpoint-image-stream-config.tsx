import type { ProviderEndpoint } from 'src/types/provider';

import Switch from '@mui/material/Switch';
import FormGroup from '@mui/material/FormGroup';
import FormControlLabel from '@mui/material/FormControlLabel';

import { useTranslate } from 'src/locales/use-locales';

const IMAGE_STREAM_MODE_KEY = 'upstream_image_stream_mode';
const IMAGE_STREAM_MODE_NATIVE = 'native_stream';
const IMAGE_STREAM_MODE_SYNC_WRAPPED = 'sync_wrapped_stream';

const IMAGE_API_FORMATS = new Set([
  'openai_image',
  'openai:image',
  'openai:image_generation',
  'openai_image_edit',
  'openai:image_edit',
]);

export function isOpenAiImageEndpointFormat(apiFormat: string) {
  return IMAGE_API_FORMATS.has(apiFormat.trim().toLowerCase());
}

export function endpointNativeImageStreamEnabled(endpoint: ProviderEndpoint) {
  return endpoint.format_acceptance_config?.[IMAGE_STREAM_MODE_KEY] === IMAGE_STREAM_MODE_NATIVE;
}

export function imageStreamConfigFromNativeEnabled(enabled: boolean) {
  return {
    [IMAGE_STREAM_MODE_KEY]: enabled ? IMAGE_STREAM_MODE_NATIVE : IMAGE_STREAM_MODE_SYNC_WRAPPED,
  };
}

export function imageEndpointFormatConfig(apiFormat: string, enabled: boolean) {
  return isOpenAiImageEndpointFormat(apiFormat) ? imageStreamConfigFromNativeEnabled(enabled) : null;
}

export function formatConfigChanged(endpoint: ProviderEndpoint, enabled: boolean) {
  if (!isOpenAiImageEndpointFormat(endpoint.api_format)) return false;
  return endpointNativeImageStreamEnabled(endpoint) !== enabled;
}

export function ProviderEndpointImageStreamSwitch({
  visible,
  checked,
  onChange,
}: {
  visible: boolean;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const { t } = useTranslate('admin');
  if (!visible) return null;
  return (
    <FormGroup>
      <FormControlLabel
        control={<Switch checked={checked} onChange={(event) => onChange(event.target.checked)} />}
        label={t('providers.upstreamImageNativeStream')}
      />
    </FormGroup>
  );
}
