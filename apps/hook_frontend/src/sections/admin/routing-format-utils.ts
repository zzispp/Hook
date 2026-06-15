import type { RouteIdentity } from 'src/types/routing';

type RouteFormatSource = Pick<RouteIdentity, 'client_api_format' | 'provider_api_format'>;

export function formatRouteApiFormat(route: RouteFormatSource) {
  if (!route.provider_api_format || route.provider_api_format === route.client_api_format) {
    return route.client_api_format;
  }

  return `${route.client_api_format} -> ${route.provider_api_format}`;
}

export function routeNeedsConversion(route: RouteFormatSource) {
  return Boolean(
    route.provider_api_format && route.provider_api_format !== route.client_api_format
  );
}
