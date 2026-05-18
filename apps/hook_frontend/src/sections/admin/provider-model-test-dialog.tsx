'use client';

import type { ProviderEndpoint, ProviderModelBinding, ProviderModelTestResponse } from 'src/types/provider';

import { useMemo, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/locales/use-locales';
import { testProviderModel } from 'src/actions/providers';

import { toast } from 'src/components/snackbar';
import { Iconify } from 'src/components/iconify';

import { formatApiFormat } from './provider-management-utils';
import { RequestRecordJsonViewer } from './request-record-json-viewer';
import { resultSx, editorGridSx, editorInputSx, endpointGridSx, endpointButtonSx } from './provider-model-test-styles';
import {
  formatJsonDraft,
  defaultModelTestBody,
  parseJsonObjectDraft,
  defaultModelTestHeaders,
} from './provider-model-test-utils';

const TESTABLE_FORMATS = ['openai_chat', 'openai_cli', 'openai_compact', 'claude_chat', 'gemini_cli'];

type Props = {
  providerId: string;
  binding: ProviderModelBinding | null;
  endpoints: ProviderEndpoint[];
  onClose: () => void;
};

export function ProviderModelTestDialog({ providerId, binding, endpoints, onClose }: Props) {
  const { t } = useTranslate('admin');
  const activeEndpoints = useMemo(() => testableEndpoints(endpoints), [endpoints]);
  const [endpointId, setEndpointId] = useState('');
  const [headersDraft, setHeadersDraft] = useState(defaultModelTestHeaders);
  const [bodyDraft, setBodyDraft] = useState('{}');
  const [testing, setTesting] = useState(false);
  const [result, setResult] = useState<ProviderModelTestResponse | null>(null);
  const selectedEndpoint = useMemo(
    () => activeEndpoints.find((endpoint) => endpoint.id === endpointId),
    [activeEndpoints, endpointId]
  );

  useEffect(() => {
    if (!binding) return;
    const endpoint = activeEndpoints[0];
    setEndpointId(endpoint?.id ?? '');
    setHeadersDraft(defaultModelTestHeaders());
    setBodyDraft(defaultModelTestBody(binding, endpoint?.api_format));
    setResult(null);
  }, [activeEndpoints, binding]);

  const handleEndpointSelect = (id: string) => {
    setEndpointId(id);
    const endpoint = activeEndpoints.find((item) => item.id === id);
    if (binding && endpoint) {
      setBodyDraft(defaultModelTestBody(binding, endpoint.api_format));
    }
    setResult(null);
  };

  const submit = async () => {
    if (!binding || testing) return;
    const payload = parsePayload(endpointId, headersDraft, bodyDraft);
    if (payload.error) {
      toast.error(payload.error);
      return;
    }
    setTesting(true);
    setResult(null);
    try {
      const response = await testProviderModel(providerId, binding.id, payload.value);
      setResult(response);
      toast[response.success ? 'success' : 'error'](
        response.success ? t('messages.providerModelTestSucceeded') : response.error || t('messages.providerModelTestFailed')
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.providerModelTestFailed'));
    } finally {
      setTesting(false);
    }
  };

  return (
    <Dialog fullWidth maxWidth="md" open={Boolean(binding)} onClose={testing ? undefined : onClose}>
      <DialogTitle>
        {t('providers.modelTestTitle')}
        <Typography variant="body2" color="text.secondary" sx={{ mt: 0.5 }}>
          {binding ? t('providers.modelTestDescription', { name: binding.provider_model_name }) : ''}
        </Typography>
      </DialogTitle>
      <DialogContent dividers sx={{ px: 3, py: 2 }}>
        <Stack spacing={2.5}>
          <EndpointPicker endpoints={activeEndpoints} selectedId={endpointId} onSelect={handleEndpointSelect} />
          <EditorGrid
            headersDraft={headersDraft}
            bodyDraft={bodyDraft}
            onHeadersChange={setHeadersDraft}
            onBodyChange={setBodyDraft}
            onFormatHeaders={() => setHeadersDraft(formatJsonDraft(headersDraft))}
            onFormatBody={() => setBodyDraft(formatJsonDraft(bodyDraft))}
            onResetHeaders={() => setHeadersDraft(defaultModelTestHeaders())}
            onResetBody={() => binding && setBodyDraft(defaultModelTestBody(binding, selectedEndpoint?.api_format))}
          />
          {result ? <TestResult result={result} /> : null}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose} disabled={testing}>{t('common.cancel')}</Button>
        <Button variant="contained" loading={testing} disabled={!binding || !endpointId} onClick={submit}>
          {t('providers.startModelTest')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function EndpointPicker({
  endpoints,
  selectedId,
  onSelect,
}: {
  endpoints: ProviderEndpoint[];
  selectedId: string;
  onSelect: (id: string) => void;
}) {
  const { t } = useTranslate('admin');
  if (endpoints.length === 0) {
    return <Typography variant="body2" color="text.secondary">{t('providers.noTestableEndpoints')}</Typography>;
  }
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('providers.selectTestEndpoint')}</Typography>
      <Box sx={endpointGridSx}>
        {endpoints.map((endpoint) => (
          <Button
            key={endpoint.id}
            variant={selectedId === endpoint.id ? 'soft' : 'outlined'}
            color={selectedId === endpoint.id ? 'primary' : 'inherit'}
            onClick={() => onSelect(endpoint.id)}
            sx={endpointButtonSx}
          >
            <Box sx={{ minWidth: 0, textAlign: 'left' }}>
              <Typography variant="subtitle2" noWrap>{formatApiFormat(endpoint.api_format)}</Typography>
              <Typography variant="caption" color="text.secondary" sx={{ wordBreak: 'break-all' }}>
                {endpoint.base_url}
              </Typography>
            </Box>
            <Chip size="small" label={selectedId === endpoint.id ? t('providers.selected') : t('common.enabled')} />
          </Button>
        ))}
      </Box>
    </Stack>
  );
}

function EditorGrid(props: {
  headersDraft: string;
  bodyDraft: string;
  onHeadersChange: (value: string) => void;
  onBodyChange: (value: string) => void;
  onFormatHeaders: () => void;
  onFormatBody: () => void;
  onResetHeaders: () => void;
  onResetBody: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={editorGridSx}>
      <JsonEditor title={t('providers.testRequestHeaders')} value={props.headersDraft} onChange={props.onHeadersChange} onFormat={props.onFormatHeaders} onReset={props.onResetHeaders} />
      <JsonEditor title={t('providers.testRequestBody')} value={props.bodyDraft} onChange={props.onBodyChange} onFormat={props.onFormatBody} onReset={props.onResetBody} />
    </Box>
  );
}

function JsonEditor(props: {
  title: string;
  value: string;
  onChange: (value: string) => void;
  onFormat: () => void;
  onReset: () => void;
}) {
  return (
    <Stack spacing={1}>
      <Stack direction="row" alignItems="center" justifyContent="space-between">
        <Typography variant="subtitle2">{props.title}</Typography>
        <Stack direction="row" spacing={0.5}>
          <IconButton size="small" onClick={props.onFormat}>
            <Iconify icon="solar:code-bold" width={16} />
          </IconButton>
          <IconButton size="small" onClick={props.onReset}>
            <Iconify icon="solar:restart-bold" width={16} />
          </IconButton>
        </Stack>
      </Stack>
      <TextField
        fullWidth
        multiline
        minRows={12}
        value={props.value}
        onChange={(event) => props.onChange(event.target.value)}
        slotProps={{ input: { sx: editorInputSx } }}
      />
    </Stack>
  );
}

function TestResult({ result }: { result: ProviderModelTestResponse }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={resultSx(result.success)}>
      <Stack spacing={1.5}>
        <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap" useFlexGap>
          <Chip color={result.success ? 'success' : 'error'} label={result.success ? t('providers.testSucceeded') : t('providers.testFailed')} />
          <Chip label={`${t('providers.statusCode')}: ${result.status_code ?? '-'}`} />
          <Chip label={`${t('providers.latency')}: ${result.latency_ms}ms`} />
        </Stack>
        {result.error ? <Typography variant="body2" color="error">{result.error}</Typography> : null}
        <ResultBlock title={t('providers.requestUrl')} value={result.request_url} />
        <ResultBlock title={t('providers.upstreamRequestBody')} value={result.request_body} />
        <ResultBlock title={t('providers.upstreamResponseBody')} value={result.response_body} />
      </Stack>
    </Box>
  );
}

function ResultBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <Stack spacing={0.75}>
      <Typography variant="caption" color="text.secondary">{title}</Typography>
      <RequestRecordJsonViewer value={value} />
    </Stack>
  );
}

function parsePayload(endpointId: string, headersDraft: string, bodyDraft: string) {
  if (!endpointId) {
    return { value: null as never, error: '请选择要测试的端点' };
  }
  const headers = parseJsonObjectDraft(headersDraft, {});
  if (headers.error || !headers.value) {
    return { value: null as never, error: `测试请求头无效: ${headers.error || 'Invalid JSON'}` };
  }
  const body = parseJsonObjectDraft(bodyDraft, null);
  if (body.error || !body.value) {
    return { value: null as never, error: `测试请求体无效: ${body.error || 'Invalid JSON'}` };
  }
  return { value: { endpoint_id: endpointId, request_headers: stringHeaders(headers.value), request_body: body.value }, error: null };
}

function stringHeaders(headers: Record<string, unknown>) {
  return Object.fromEntries(
    Object.entries(headers)
      .filter(([key, value]) => key.trim() && value !== null && value !== undefined)
      .map(([key, value]) => [key.trim(), typeof value === 'string' ? value : JSON.stringify(value)])
  );
}

function testableEndpoints(endpoints: ProviderEndpoint[]) {
  return endpoints.filter((endpoint) => endpoint.is_active && TESTABLE_FORMATS.includes(endpoint.api_format));
}
