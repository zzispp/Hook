export function logoImageSource(value: string) {
  const logo = value.trim();
  if (!logo) {
    return '';
  }

  return logo.startsWith('data:') ? logo : `data:image/svg+xml;base64,${logo}`;
}
