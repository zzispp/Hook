'use client';

import type { ApiEndpoint } from 'src/types/system-setting';
import type { SystemSettingsForm } from './system-settings-utils';

export type ApiEndpointSetForm = React.Dispatch<React.SetStateAction<SystemSettingsForm>>;

export function addApiEndpoint(setForm: ApiEndpointSetForm) {
  setForm((current) => ({
    ...current,
    api_endpoints: [...current.api_endpoints, defaultApiEndpoint()],
  }));
}

export function updateApiEndpoint(
  setForm: ApiEndpointSetForm,
  index: number,
  endpoint: ApiEndpoint
) {
  setForm((current) => ({
    ...current,
    api_endpoints: current.api_endpoints.map((item, itemIndex) =>
      itemIndex === index ? endpoint : item
    ),
  }));
}

export function removeApiEndpoint(setForm: ApiEndpointSetForm, index: number) {
  setForm((current) => ({
    ...current,
    api_endpoints: current.api_endpoints.filter((_item, itemIndex) => itemIndex !== index),
  }));
}

export function moveApiEndpoint(
  setForm: ApiEndpointSetForm,
  index: number,
  direction: -1 | 1
) {
  setForm((current) => {
    const next = [...current.api_endpoints];
    const target = index + direction;
    [next[index], next[target]] = [next[target], next[index]];
    return { ...current, api_endpoints: next };
  });
}

function defaultApiEndpoint(): ApiEndpoint {
  return {
    id: crypto.randomUUID(),
    name: '',
    url: '',
    description: '',
  };
}
