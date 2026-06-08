const DEFAULT_FULL_LOGO_SOURCE = '/logo/logo.svg';
const DEFAULT_ICON_LOGO_SOURCE = '/logo/logo-icon.svg';

export function logoImageSource(value: string) {
  const logo = value.trim();
  if (!logo) {
    return '';
  }

  return isDirectLogoSource(logo) ? logo : `data:image/svg+xml;base64,${logo}`;
}

export function logoDisplaySource(value: string, options: { isSingle: boolean }) {
  const source = logoImageSource(value);
  if (options.isSingle && source === DEFAULT_FULL_LOGO_SOURCE) {
    return DEFAULT_ICON_LOGO_SOURCE;
  }

  return source;
}

export function isMaskableLogoSource(value: string) {
  return value === DEFAULT_FULL_LOGO_SOURCE || value === DEFAULT_ICON_LOGO_SOURCE;
}

function isDirectLogoSource(value: string) {
  return value.startsWith('data:') || value.startsWith('/');
}
