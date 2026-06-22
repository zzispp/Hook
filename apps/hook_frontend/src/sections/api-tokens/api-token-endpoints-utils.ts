import type { ApiEndpoint, PublicSiteInfo } from 'src/types/system-setting';

export type DisplayApiEndpoint = ApiEndpoint & {
  isDefault: boolean;
};

export function effectiveApiEndpoints(
  site: PublicSiteInfo | undefined,
  defaultName: string
): DisplayApiEndpoint[] {
  const configured = normalizedConfiguredEndpoints(site?.api_endpoints ?? []);
  if (configured.length > 0) {
    return configured.map((endpoint) => ({ ...endpoint, isDefault: false }));
  }

  const publicBaseUrl = normalizeEndpointUrl(site?.public_base_url ?? '');
  if (!publicBaseUrl) {
    return [];
  }

  return [
    {
      id: 'public_base_url',
      name: defaultName,
      url: publicBaseUrl,
      description: '',
      isDefault: true,
    },
  ];
}

export function endpointUrl(baseUrl: string, path: string) {
  return new URL(trimmedPath(path), withTrailingSlash(normalizeEndpointUrl(baseUrl))).toString();
}

export function normalizeEndpointUrl(value: string) {
  return value.trim().replace(/\/+$/, '');
}

function normalizedConfiguredEndpoints(endpoints: ApiEndpoint[]) {
  return endpoints.map((endpoint) => ({
    ...endpoint,
    id: endpoint.id.trim(),
    name: endpoint.name.trim(),
    url: normalizeEndpointUrl(endpoint.url),
    description: endpoint.description.trim(),
  }));
}

function trimmedPath(path: string) {
  return path.startsWith('/') ? path.slice(1) : path;
}

function withTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}
